mod lexer;
mod ast;
mod syntax;

use clap::{AppSettings, Clap};
use ::syntax::Parseable;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
	#[clap(short, long)]
	verbose: bool,

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

	#[clap(long)]
	triple: Option<String>,

	#[clap(long)]
	emit_ast: bool
}

fn build(build_opts: &BuildOpts, verbose: bool) {
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

	if verbose { println!("Parsed"); }

	if build_opts.emit_ast {
		println!("{:#?}", unit);
	}
}

fn main() {
	let opts = Opts::parse();

	match opts.cmd {
		SubCommand::Build(build_opts) => {
			build(&build_opts, opts.verbose)
		}
	}
}
