mod lexer;
mod ast;
mod irgen;

use std::path::{Path, PathBuf};

use clap::{AppSettings, Clap};
use ir2triple;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    cmd: SubCommand
}

#[derive(Clap)]
enum SubCommand {
    Build(BuildOpts)
}

#[derive(Clap, Debug)]
struct BuildOpts {
    path: Vec<String>,

    #[clap(short='I')]
    include: Vec<String>,

    #[clap(short, long, default_value = "a.out")]
    output: String,

    #[clap(long, short, default_value = "linux-elf-x86_64")]
    triple: String,

    #[clap(short = 'c')]
    relocatable: bool,

    #[clap(long)]
    emit_ast: bool,
    #[clap(long)]
    emit_ir: bool,
}

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

pub struct BuildContext {
    linked_paths: Vec<PathBuf>,
    target_arch_name: &'static str,
    search_dirs: Vec<PathBuf>
}

impl BuildContext {
    pub fn new(linked_paths: &Vec<String>, target_arch_name: &'static str, search_dirs: &Vec<String>) -> BuildContext {
        BuildContext {
            linked_paths: linked_paths.iter().map(|x| Path::new(x).canonicalize().expect("Invalid path")).collect(),
            target_arch_name,
            search_dirs: search_dirs.iter().map(|x| Path::new(x).canonicalize().expect("Invalid path")).collect()
        }
    }

    fn find_import(&self, working: PathBuf, name: &Vec<String>) -> Option<PathBuf> {
        let mut child_path = working;
        for element in name {
            child_path = child_path.join(element);
        }

        child_path = child_path.with_extension("nl");
        if child_path.exists() { return Some(child_path); }

        for search_dir in &self.search_dirs {
            let mut child_path = search_dir.clone();
            for element in name {
                child_path = child_path.join(element);
            }

            child_path = child_path.with_extension("nl");
            if child_path.exists() { return Some(child_path); }            
        }

        None
    }

    fn append_at_path(&self, ir_unit: &mut ir::TranslationUnit, path: &PathBuf, visited_paths: &mut Vec<PathBuf>) {
        let path = path.canonicalize().expect("Invalid path");

        if visited_paths.iter().position(|x| x == &path).is_some() {
            return;
        }

        visited_paths.push(path.clone());

        let content = match std::fs::read_to_string(&path) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Could not open {} - {}", path.display(), e);
                std::process::exit(1);
            }
        };

        let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
        matcher.step();

        let unit = match ast::TranslationUnit::parse(&mut matcher) {
            ::syntax::MatchResult::Ok(code) => code,
            ::syntax::MatchResult::Err(e) => {
                eprintln!("SyntaxError in {}: {}", path.display(), e.message());
                print_error_range(e.start(), e.end(), &content, &path, e.message());
                std::process::exit(1);
            },
            _ => unreachable!()
        };

        for node in &unit.nodes {
            match node {
                ast::TopLevelNode::Import(import_stmt) => {
                    if let Some(child_path) = self.find_import(path.parent().unwrap().to_path_buf(), &import_stmt.path) {
                        self.append_at_path(ir_unit, &child_path, visited_paths);
                    } else {
                        let error = format!("Could not resolve import {}", import_stmt.path.join("."));
                        eprintln!("ImportError in {}: Could not resolve import {}", path.display(), error);
                        print_error_range(import_stmt.span.start, import_stmt.span.end, &content, &path, &error);
                        std::process::exit(1);
                    }
                },
                _ => {}
            }
        }

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

    pub fn build(&mut self) -> ir::TranslationUnit {
        let mut ir_unit = ir::TranslationUnit::new();

        let mut visited_paths = Vec::new();
        for path in &self.linked_paths {
            self.append_at_path(&mut ir_unit, path, &mut visited_paths);
        }

        ir_unit
    }
}

enum Arch {
    X86,
    Wasm,
    Java,
    None
}

impl Arch {
    pub fn parse_triple(triple: &str) -> Option<Arch> {
        match triple {
            "linux-elf-x86_64" => Some(Arch::X86),
            "wasm" => Some(Arch::Wasm),
            "java" => Some(Arch::Java),
            "none" => Some(Arch::None),
            _ => None
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Arch::X86 => "x86",
            Arch::Wasm => "wasm",
            Arch::Java => "java",
            Arch::None => "none",
        }
    }

    pub fn encode(&self, ir_unit: &ir::TranslationUnit, path: &str, relocatable: bool) -> Result<(), String> {
        match self {
            Arch::X86 => ir2triple::linux_elf::encode(&ir_unit, path, relocatable) ,
            Arch::Wasm => ir2triple::wasm::encode(&ir_unit, path, relocatable),
            Arch::Java => ir2triple::java::encode(&ir_unit, path, relocatable),
            Arch::None => Ok(()) // Do nothing
        }
    }
}

fn build(build_opts: &BuildOpts) {
    let arch = match Arch::parse_triple(&build_opts.triple) {
        Some(a) => a,
        None => {
            println!("Unknown triple: {}", build_opts.triple);
            std::process::exit(1);
        }
    };
    let ir_unit = BuildContext::new(&build_opts.path, arch.short_name(), &build_opts.include).build();

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
                        ir::FunctionAttr::ExternLocation(location) => print!("@extern({:?})", location),
                    }
                }
            }
            println!(" {}", func);
        }
    }

    ir_unit.validate().expect("Could not validate IR");

    match arch.encode(&ir_unit, &build_opts.output, build_opts.relocatable) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("EncodeError: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let opts = Opts::parse();

    match opts.cmd {
        SubCommand::Build(build_opts) => build(&build_opts)
    }
}
