mod lexer;
mod ast;
mod irgen;

use std::path::{Path, PathBuf};

use clap::{AppSettings, Clap};
use ir2triple;

// Uses https://docs.rs/clap to parse command line arguments

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    Build(BuildOpts),
    Run(BuildOpts)
}

#[derive(Clap, Debug)]
struct BuildOpts {
    /// Paths to root source file
    path: Vec<String>,

    /// Add directories to import search path
    #[clap(short='I')]
    include: Vec<String>,

    /// Set name of output binary (or main class, in the case of Java)
    #[clap(short, long, default_value = "a.out")]
    output: String,

    /// Target triple. Valid values are linux-elf-x86_64, macos-macho-x86_64, wasm, java, none and native - which infers the type from the calling system.
    #[clap(long, short, default_value = "native")]
    triple: String,

    /// Make the target file relocatable.
    #[clap(short = 'c')]
    relocatable: bool,

    /// Link the binary using CC (only applies to linux-elf-x86_64 on linux-elf-x86_64 systems)
    #[clap(long)]
    link: bool,

    /// Include std in the generated binary
    #[clap(long)]
    std: bool,

    /// Add files to pass to CC, only applies if --link is enabled
    #[clap(short='T')]
    ldinc: Vec<String>,

    /// Emit the parsed AST to stdout for each file after it is parsed
    #[clap(long)]
    emit_ast: bool,

    // Emit the generated IR to stdout
    #[clap(long)]
    emit_ir: bool,
}

/// Pretty prints an error to stderr caused at the given location, with a message.
fn print_error_range(mut start: usize, mut end: usize, source: &str, path: &Path, message: &str) {
    while start < source.len() - 1 && source.as_bytes()[start].is_ascii_whitespace() {
        start += 1;
    }

    while end > 0 && source.as_bytes()[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    
    end = end.max(start);

    let mut idx = 0;
    let main_line = source[0..start].matches('\n').count();
    let first_line = if main_line <= 3 { 0 } else { main_line - 3 };
    
    eprintln!("\u{001b}[34m{}:{}\u{001b}[0m", path.display(), main_line);
    for (l, line) in source.lines().enumerate() {
        if l < first_line || l > main_line + 3 {
            idx += line.len() + 1;
            continue;
        }

        eprint!("\u{001b}[34m{:>4}\u{001b}[0m  ", l + 1);
        if l == main_line {
            eprintln!("{}\u{001b}[31m{}\u{001b}[0m{}", &source[idx..start], &source[start..end], &source[end..idx + line.len()]);
            eprintln!("{}\u{001b}[33m^ {} here\u{001b}[0m", " ".repeat(source[idx..start].len() + 6), message);
        } else {
            eprintln!("{}", line);
        }

        idx += line.len() + 1;
    }
}

/// Inheritted from NL_ROOT environment variable e.g. (NL_ROOT=/usr/lib/nl).
fn env_search_dir() -> Option<PathBuf> {
    let var = match std::env::var("NL_ROOT") {
        Ok(var) => var,
        Err(_) => return None
    };

    match PathBuf::from(var).canonicalize() {
        Ok(path) => Some(path),
        Err(_) => None
    }
}

fn env_search_dir_with(name: &str) -> Option<PathBuf> {
    let mut path = env_search_dir()?;
    path.push(name);
    Some(path)
}

/// Characterises the build system for NL, primarily is concerned with resolving imports and constructing one large IR translation unit
pub struct BuildContext {
    linked_paths: Vec<PathBuf>,
    target_arch_name: &'static str,
    search_dirs: Vec<PathBuf>,
    emit_ast: bool,
    env_search_dir: Option<PathBuf>
}

impl BuildContext {
    pub fn new(linked_paths: &Vec<String>, target_arch_name: &'static str, search_dirs: &Vec<String>, emit_ast: bool) -> BuildContext {
        BuildContext {
            linked_paths: linked_paths.iter().map(|x| Path::new(x).canonicalize().expect("Invalid path")).collect(),
            target_arch_name,
            search_dirs: search_dirs.iter().map(|x| Path::new(x).canonicalize().expect("Invalid path")).collect(),
            emit_ast,
            env_search_dir: env_search_dir()
        }
    }

    pub fn append_linked_path(&mut self, path: PathBuf) {
        self.linked_paths.push(path.canonicalize().expect("Invalid path"));
    }

    /// Searches for an import, or None if none is found.
    fn find_import(&self, working: PathBuf, name: &Vec<String>) -> Option<PathBuf> {
        // Construct a path relative to the working directory by joining the import components and adding '.nl., e.g. 'import a.b.c' -> 'a/b/c.nl' (or 'a\b\c.nl' for windows)
        let mut child_path = working;
        for element in name {
            child_path = child_path.join(element);
        }

        child_path = child_path.with_extension("nl");
        if child_path.exists() { return Some(child_path); }

        // If that fails, try the same for self.search_dirs
        for search_dir in &self.search_dirs {
            let mut child_path = search_dir.clone();
            for element in name {
                child_path = child_path.join(element);
            }

            child_path = child_path.with_extension("nl");
            if child_path.exists() { return Some(child_path); }            
        }

        // If that fails, try the distrubution constant
        if let Some(ref dir) = self.env_search_dir {
            let mut child_path = dir.clone();

            for element in name {
                child_path = child_path.join(element);
            }
    
            child_path = child_path.with_extension("nl");
            if child_path.exists() { return Some(child_path); }            
        }

        // No matching file was found
        None
    }

    /// Parses and does IRGen for the given path, pushing the result to ir_unit.
    fn append_at_path(&self, ir_unit: &mut ir::TranslationUnit, path: &PathBuf, visited_paths: &mut Vec<PathBuf>) {
        let path = path.canonicalize().expect("Invalid path");

        // Check we have not already processed this path - this prevents infinite import loops
        if visited_paths.iter().position(|x| x == &path).is_some() {
            return;
        }
        visited_paths.push(path.clone()); // Pushed before we start parsing, otherwise doesn't prevent loops

        let content = match std::fs::read_to_string(&path) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Could not open {} - {}", path.display(), e);
                std::process::exit(1);
            }
        };

        let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
        matcher.step(); // Focus on the first token

        let unit = match ast::TranslationUnit::parse(&mut matcher) {
            ::syntax::MatchResult::Ok(code) => code,
            ::syntax::MatchResult::Err(e) => {
                eprintln!("SyntaxError in {}: {}", path.display(), e.message());
                print_error_range(e.start(), e.end(), &content, &path, e.message());
                std::process::exit(1);
            },
            _ => unreachable!()
        };

        if self.emit_ast {
            println!("{:#?}", unit);
        }

        // Resolve imports
        for node in &unit.nodes {
            match node {
                ast::TopLevelNode::Import(import_stmt) => {
                    if let Some(child_path) = self.find_import(path.parent().unwrap().to_path_buf(), &import_stmt.path) {
                        self.append_at_path(ir_unit, &child_path, visited_paths);
                    } else {
                        let error = format!("Could not resolve import {}", import_stmt.path.join("."));
                        eprintln!("ImportError in {}: {}", path.display(), error);
                        print_error_range(import_stmt.span.start, import_stmt.span.end, &content, &path, &error);
                        std::process::exit(1);
                    }
                },
                _ => {}
            }
        }

        // If this file is `linked` (i.e. it was in the list of source files given on the command line) then the actual code of functions needs to be added to the IR unit.
        // Otherwise, all functions can be made extern.
        if self.linked_paths.iter().position(|x| x == &path).is_some() {
            match unit.to_ir_on(ir_unit, self.target_arch_name) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("SemanticError: {}: {}", path.display(), e.message());
                    print_error_range(e.start(), e.end(), &content, &path, &e.message());
                    std::process::exit(1);
                }
            }
        } else {
            match unit.to_extern_ir_on(ir_unit, self.target_arch_name) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("SemanticError: {}: {}", path.display(), e.message());
                    print_error_range(e.start(), e.end(), &content, &path, &e.message());
                    std::process::exit(1);
                }
            }
        }
    }

    /// Root build method of BuildContext, will create, populate and return the TranslationUnit
    pub fn build(&mut self) -> ir::TranslationUnit {
        let mut ir_unit = ir::TranslationUnit::new();

        let mut visited_paths = Vec::new();
        for path in &self.linked_paths {
            self.append_at_path(&mut ir_unit, path, &mut visited_paths);
        }

        ir_unit
    }
}

/// Represents the possible triples provided by the user on the command line
enum Arch {
    LinuxX86,
    MacosX86,
    Wasm,
    Java,
    None
}

impl Arch {
    /// Convert from a string triple name to an Arch, or None otherwise
    pub fn parse_triple(triple: &str) -> Option<Arch> {
        match triple {
            "linux-elf-x86_64" => Some(Arch::LinuxX86),
            "macos-macho-x86_64" => Some(Arch::MacosX86),
            "wasm" => Some(Arch::Wasm),
            "java" => Some(Arch::Java),
            "none" => Some(Arch::None),
            "native" =>
                if cfg!(target_os = "macos") {
                    Some(Arch::MacosX86)
                } else {
                    Some(Arch::LinuxX86)
                },
            _ => None
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Arch::LinuxX86 => "linux-x86",
            Arch::MacosX86 => "macos-x86",
            Arch::Wasm => "wasm",
            Arch::Java => "java",
            Arch::None => "none",
        }
    }

    fn link(tmp: &PathBuf, build_opts: &BuildOpts) -> Result<(), String> {
        if build_opts.std {
            match std::process::Command::new("cc")
                .args(&build_opts.ldinc)
                .arg(tmp)
                .arg(env_search_dir_with("std.c").expect("No NL_ROOT"))
                .arg("-o").arg(&build_opts.output)
                .status() {
                Ok(_) => Ok(()),
                Err(err) => return Err(format!("{}", err))
            }
        } else {
            match std::process::Command::new("cc")
                .args(&build_opts.ldinc)
                .arg(tmp)
                .arg("-o").arg(&build_opts.output)
                .status() {
                Ok(_) => Ok(()),
                Err(err) => return Err(format!("{}", err))
            }
        }
    }

    /// Invokes the relevant ir2triple module for this triple
    pub fn encode(&self, ir_unit: &ir::TranslationUnit, build_opts: &BuildOpts) -> Result<(), String> {
        match self {
            Arch::LinuxX86 if build_opts.link && build_opts.relocatable => {
                let mut tmp = std::env::temp_dir();
                tmp.push("nl-build.o");
                ir2triple::linux_elf::encode(&ir_unit, tmp.to_str().unwrap(), true)?;

                Arch::link(&tmp, build_opts)?;

                match std::fs::remove_file(tmp) {
                    Ok(_) => {},
                    Err(e) => return Err(format!("{}", e))
                }
                
                Ok(())
            },
            Arch::LinuxX86 => ir2triple::linux_elf::encode(&ir_unit, &build_opts.output, build_opts.relocatable),
            Arch::MacosX86 if build_opts.link && build_opts.relocatable => {
                let mut tmp = std::env::temp_dir();
                tmp.push("nl-build.o");
                ir2triple::macos_macho::encode(&ir_unit, tmp.to_str().unwrap(), true)?;

                Arch::link(&tmp, build_opts)?;

                match std::fs::remove_file(tmp) {
                    Ok(_) => {},
                    Err(e) => return Err(format!("{}", e))
                }
                
                Ok(())
            },
            Arch::MacosX86 => ir2triple::macos_macho::encode(&ir_unit, &build_opts.output, build_opts.relocatable),
            Arch::Wasm => ir2triple::wasm::encode(&ir_unit, &build_opts.output, build_opts.relocatable),
            Arch::Java => ir2triple::java::encode(&ir_unit, &build_opts.output, build_opts.relocatable),
            Arch::None => Ok(()) // Do nothing
        }
    }

    pub fn run(&self, build_opts: &BuildOpts) -> Result<(), String> {
        match self {
            Arch::LinuxX86 | Arch::MacosX86 => {
                match std::process::Command::new(&PathBuf::from(&build_opts.output).canonicalize().unwrap())
                    .status() {
                        Ok(code) => {
                            if !code.success() {
                                println!("Process exitted with code {}", code);
                            }
                            Ok(())
                        }
                        Err(err) =>  Err(format!("{}", err))
                }
            },
            Arch::Wasm => {
                match std::process::Command::new("node")
                    .arg(&env_search_dir_with("wasm.js").expect("No NL_ROOT"))
                    .arg(&build_opts.output)
                    .status() {
                        Ok(code) => {
                            if !code.success() {
                                println!("Process exitted with code {}", code);
                            }
                            Ok(())
                        }
                        Err(err) =>  Err(format!("{}", err))
                }
            },
            Arch::Java => {
                let classpath = PathBuf::from(&build_opts.output);

                match std::process::Command::new("java")
                    .arg(&classpath.file_stem().unwrap())
                    .status() {
                        Ok(code) => {
                            if !code.success() {
                                println!("Process exitted with {}", code);
                            }
                            Ok(())   
                        }
                        Err(err) =>  Err(format!("{}", err))
                }
            },
            Arch::None => {
                eprintln!("Nothing to run");
                Ok(()) // Do nothing
            }
        }
    }
}

/// Entry point of the build subcommand
fn build_and_run(build_opts: &BuildOpts, run: bool) {
    // Parse the triple, or fail of it is invalid
    let arch = match Arch::parse_triple(&build_opts.triple) {
        Some(a) => a,
        None => {
            println!("Unknown triple: {}", build_opts.triple);
            std::process::exit(1);
        }
    };

    // Parse and build the IR Unit
    let mut ctx = BuildContext::new(&build_opts.path, arch.short_name(), &build_opts.include, build_opts.emit_ast);
    if build_opts.std {
        ctx.append_linked_path(PathBuf::from(env_search_dir_with("std.nl").expect("No NL_ROOT")));
    }

    let ir_unit = ctx.build();

    if build_opts.emit_ir {
        for (idx, func) in ir_unit.functions().iter().enumerate() {
            print!("func {}:{:?}", idx, func.name());
            if func.attr_count() != 0 {
                for attr in func.attrs() {
                    print!(" ");
                    match attr {
                        ir::FunctionAttr::Entry => print!("@entry"),
                        ir::FunctionAttr::Alloc => print!("@alloc"),
                        ir::FunctionAttr::AllocSlice => print!("@alloc_slice"),
                        ir::FunctionAttr::Free => print!("@free"),
                        ir::FunctionAttr::FreeSlice => print!("@free_slice"),
                        ir::FunctionAttr::ExternLocation(location) => print!("@extern({:?})", location),
                    }
                }
            }
            println!(" {}", func);
        }
    }

    // Does not *strictly* need to be here, but good for debugging
    ir_unit.validate().expect("Could not validate IR");

    match arch.encode(&ir_unit, &build_opts) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("EncodeError: {}", e);
            std::process::exit(1);
        }
    }

    if run {
        match arch.run(&build_opts) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("RunError: {}", e);
                std::process::exit(1);
            }   
        }
    }
}

fn main() {
    let opts = Opts::parse();

    match opts.cmd {
        SubCommand::Build(build_opts) => build_and_run(&build_opts, false),
        SubCommand::Run(build_opts) => build_and_run(&build_opts, true),
    }
}
