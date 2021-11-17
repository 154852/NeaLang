mod lexer;
mod ast;
mod syntax;
mod irgen;

use std::path::Path;

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

fn print_error_range(start: usize, end: usize, source: &str) {
	let mut start_idx = start;
	for _ in 0..3 {
		start_idx -= 1;
		while start_idx != 0 && source.as_bytes()[start_idx] != '\n' as u8 {
			start_idx -= 1;
		}

		if start_idx == 0 { break }
	}

	let mut end_idx = end;
	for _ in 0..3 {
		end_idx += 1;
		while end_idx != source.len() - 1 && source.as_bytes()[end_idx] != '\n' as u8 {
			end_idx += 1;
		}

		if end_idx == source.len() - 1 { break }
	}

	eprintln!("{}", "-".repeat(50));
	eprint!("{}", &source[start_idx..start]);
	eprint!("\u{001b}[31m{}\u{001b}[0m", &source[start..end]);
	eprint!("{}", &source[end..end_idx]);
	eprintln!("{}", "-".repeat(50));
}

fn append_imports_of(unit: &ast::TranslationUnit, ir_unit: &mut ir::TranslationUnit, path: &Path, relocatable: bool) {
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
					Err(e) => {
						eprintln!("Could not open import of {}: {} - {}", path.display(), child_path.display(), e);
						std::process::exit(1);
					}
				};

				let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
				matcher.step();

				let unit = match ast::TranslationUnit::parse(&mut matcher) {
					::syntax::MatchResult::Ok(code) => code,
					::syntax::MatchResult::Err(e) => {
						eprintln!("SyntaxError in {}: {}", child_path.display(), e.message());
						print_error_range(e.start(), e.end(), &content);
						std::process::exit(1);
					},
					_ => unreachable!()
				};

				append_imports_of(&unit, ir_unit, child_path.as_path(), relocatable);

				if relocatable {
					match unit.to_extern_ir_on(ir_unit) {
						Ok(_) => {},
						Err(e) => {
							eprintln!("SemanticError: {}: {}-{}: {}", path.display(), e.start(), e.end(), e.message());
							std::process::exit(1);
						}
					}
				} else {
					match unit.to_ir_on(ir_unit) {
						Ok(_) => {},
						Err(e) => {
							eprintln!("SemanticError: {}: {}-{}: {}", path.display(), e.start(), e.end(), e.message());
							std::process::exit(1);
						}
					}
				}
			},
			_ => {}
		}
	}
}

fn build(build_opts: &BuildOpts) {
	let mut ir_unit = ir::TranslationUnit::new();

	for path in &build_opts.path {
		let path = Path::new(path);
		let content = match std::fs::read_to_string(path) {
			Ok(s) => s,
			Err(e) => {
				eprintln!("Could not open source: {} - {}", path.display(), e);
				std::process::exit(1);
			}
		};

		let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
		matcher.step();

		let unit = match ast::TranslationUnit::parse(&mut matcher) {
			::syntax::MatchResult::Ok(code) => code,
			::syntax::MatchResult::Err(e) => {
				eprintln!("SyntaxError in {}: {}", path.display(), e.message());
				print_error_range(e.start(), e.end(), &content);
				std::process::exit(1);
			},
			_ => unreachable!()
		};

		if build_opts.emit_ast {
			println!("{:#?}", unit);
		}

		append_imports_of(&unit, &mut ir_unit, path, build_opts.relocatable);

		match unit.to_ir_on(&mut ir_unit) {
			Ok(_) => {},
			Err(e) => {
				eprintln!("SemanticError: {}: {}-{}: {}", path.display(), e.start(), e.end(), e.message());
				std::process::exit(1);
			}
		}
	}

	if build_opts.emit_ir {
		println!("{:#?}", ir_unit);
	}

	ir_unit.validate().expect("Could not validate IR");

	match build_opts.triple.as_str() {
		"linux-elf-x86_64" => {
			match ir2triple::linux_elf::encode(&ir_unit, &build_opts.output, build_opts.relocatable) {
				Ok(_) => (),
				Err(e) => {
					eprintln!("EncodeError: {}", e);
					std::process::exit(1);
				}
			}
		},
		"wasm" => {
			match ir2triple::wasm::encode(&ir_unit, &build_opts.output, build_opts.relocatable) {
				Ok(_) => (),
				Err(e) => {
					eprintln!("EncodeError: {}", e);
					std::process::exit(1);
				}
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
		SubCommand::Build(build_opts) => build(&build_opts)
	}
}
