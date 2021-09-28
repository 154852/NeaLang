mod lexer;
mod ast;
mod syntax;

use ::syntax::Parseable;

fn build(path: &str) {
	let content = std::fs::read_to_string(path).expect(&format!("Could not open {}", path));

    let mut matcher = crate::lexer::TokenStream::new(&content, Box::new(lexer::Matcher {}));
    matcher.step();

    match ast::Code::parse(&mut matcher) {
        ::syntax::MatchResult::Ok(code) => println!("{:#?}", code),
        ::syntax::MatchResult::Err(e) => println!("SyntaxError: {}-{}: {}", e.start(), e.end(), e.message()),
        ::syntax::MatchResult::Fail => print!("Could not match function"),
    };
}

fn main() {
    // TODO: Use clap here

	let args = std::env::args().collect::<Vec<String>>();

	match args.get(1).map(|x| x.as_str()) {
		Some("build") => build(args.get(2).expect("Expected path after run")),
		_ => {
			println!("usage: {} build <path/to/source.nl>", args.get(0).unwrap());
		}
	}
}
