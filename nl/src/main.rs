mod lexer;
mod ast;
mod syntax;
mod irgen;

use clap::{AppSettings, Clap};
use ::syntax::Parseable;
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

#[derive(Clap)]
struct BuildOpts {
	path: String,

	#[clap(short, long, default_value = "a.out")]
	output: String,

	#[clap(long, default_value = "linux-elf-x86_64")]
	triple: String,

	#[clap(short = 'c')]
	relocatable: bool,

	#[clap(long)]
	emit_ast: bool,
	#[clap(long)]
	emit_ir: bool,
}

fn build(build_opts: &BuildOpts) {
	let content = std::fs::read_to_string(&build_opts.path).expect(&format!("Could not open {}", build_opts.path));

    let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
    matcher.step();

    let unit = match ast::TranslationUnit::parse(&mut matcher) {
        ::syntax::MatchResult::Ok(code) => code,
        ::syntax::MatchResult::Err(e) => {
			println!("SyntaxError: {}-{}: {}", e.start(), e.end(), e.message());
			std::process::exit(1);
		},
		_ => unreachable!()
    };

	if build_opts.emit_ast {
		println!("{:#?}", unit);
	}

	let ir_unit = match unit.to_ir() {
		Ok(x) => x,
		Err(e) => {
			println!("SemanticError: {}-{}: {}", e.start(), e.end(), e.message());
			std::process::exit(1);
		}
	};

	if build_opts.emit_ir {
		println!("{:#?}", ir_unit);
	}

	ir_unit.validate().expect("Could not validate IR");

	match build_opts.triple.as_str() {
		"linux-elf-x86_64" => {
			ir2triple::linux_elf::encode(&ir_unit, &build_opts.output, build_opts.relocatable)
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
			build(&build_opts)
		}
	}
}
