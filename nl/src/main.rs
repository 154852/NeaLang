mod lexer;
mod ast;
mod syntax;
mod irgen;

use std::path::{Path, PathBuf};

use clap::{AppSettings, Clap};
use ir2triple;
use irgen::IrGenError;
use ::syntax::SyntaxError;

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
	path: String,

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

enum NlError {
	Syntax(SyntaxError, PathBuf),
	Semantic(IrGenError, PathBuf),
	Import(std::io::Error, PathBuf),
	Encode(String)
}

fn append_imports_of(unit: &ast::TranslationUnit, ir_unit: &mut ir::TranslationUnit, path: &Path, relocatable: bool) -> Result<(), NlError> {
	for node in &unit.nodes {
		match node {
			ast::TopLevelNode::Import(import_stmt) => {
				let mut child_path = path.parent().unwrap().to_path_buf(); // Can't be a directory, so can't be /
				for element in &import_stmt.path {
					child_path = child_path.join(element);
				}

				child_path = child_path.with_extension("nl");

				let content = match std::fs::read_to_string(&child_path) {
					Ok(x) => x,
					Err(e) => return Err(NlError::Import(e, child_path))
				};

				let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
				matcher.step();

				let unit = match ast::TranslationUnit::parse(&mut matcher) {
					::syntax::MatchResult::Ok(code) => code,
					::syntax::MatchResult::Err(e) => return Err(NlError::Syntax(e, child_path)),
					_ => unreachable!()
				};

				append_imports_of(&unit, ir_unit, child_path.as_path(), relocatable)?;

				if relocatable {
					match unit.to_extern_ir_on(ir_unit) {
						Ok(_) => {},
						Err(e) => return Err(NlError::Semantic(e, child_path))
					}
				} else {
					match unit.to_ir_on(ir_unit) {
						Ok(_) => {},
						Err(e) => return Err(NlError::Semantic(e, child_path))
					}
				}
			},
			_ => {}
		}
	}

	Ok(())
}

fn build(build_opts: &BuildOpts) -> Result<(), NlError> {
	let path = Path::new(&build_opts.path);
	let content = match std::fs::read_to_string(path) {
		Ok(s) => s,
		Err(e) => return Err(NlError::Import(e, path.to_path_buf()))
	};

    let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
    matcher.step();

    let unit = match ast::TranslationUnit::parse(&mut matcher) {
        ::syntax::MatchResult::Ok(code) => code,
        ::syntax::MatchResult::Err(e) => return Err(NlError::Syntax(e, path.to_path_buf())),
		_ => unreachable!()
    };

	if build_opts.emit_ast {
		println!("{:#?}", unit);
	}


	let mut ir_unit = ir::TranslationUnit::new();

	append_imports_of(&unit, &mut ir_unit, path, build_opts.relocatable)?;

	match unit.to_ir_on(&mut ir_unit) {
		Ok(_) => {},
		Err(e) => return Err(NlError::Semantic(e, path.to_path_buf()))
	}

	if build_opts.emit_ir {
		println!("{:#?}", ir_unit);
	}

	ir_unit.validate().expect("Could not validate IR");

	match build_opts.triple.as_str() {
		"linux-elf-x86_64" => {
			match ir2triple::linux_elf::encode(&ir_unit, &build_opts.output, build_opts.relocatable) {
				Ok(_) => Ok(()),
				Err(err) => return Err(NlError::Encode(err))
			}
		},
		"wasm" => {
			match ir2triple::wasm::encode(&ir_unit, &build_opts.output, build_opts.relocatable) {
				Ok(_) => Ok(()),
				Err(err) => return Err(NlError::Encode(err))
			}
		},
		_ => {
			println!("Unknown triple: {}", build_opts.triple);
			std::process::exit(1);
		}
	}
}

fn main() {
	let opts = Opts::parse();

	match opts.cmd {
		SubCommand::Build(build_opts) => {
			match build(&build_opts) {
				Ok(_) => {},
				Err(e) => match e {
					NlError::Syntax(e, path) => {
						eprintln!("SyntaxError: {}: {}-{}: {}", path.display(), e.start(), e.end(), e.message());
						std::process::exit(1);
					},
					NlError::Semantic(e, path) => {
						eprintln!("SemanticError: {}: {}-{}: {}", path.display(), e.start(), e.end(), e.message());
						std::process::exit(1);
					},
					NlError::Import(e, path) => {
						eprintln!("IOError: {} - {}", path.display(), e);
						std::process::exit(1);
					},
					NlError::Encode(e) => {
						eprintln!("EncodeError: {}", e);
						std::process::exit(1);
					},
				}
			}
		}
	}
}
