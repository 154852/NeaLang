use syntax::{self, MatchResult};
mod ast {
    #[derive(Debug)]
    pub struct Function {
        pub name: String,
        pub params: Vec<FunctionParam>,
        pub code: Vec<Code>
    }

    #[derive(Debug)]
    pub struct FunctionParam {
        pub name: String
    }

    #[derive(Debug)]
    pub enum Expr {
        BinaryExpr(BinaryExpr),
        Name(NameExpr),
        Closed(ClosedExpr),
        NumberLit(NumberLitExpr)
    }

    #[derive(Debug)]
    pub struct ClosedExpr {
        pub expr: Box<Expr>
    }

    #[derive(Debug)]
    pub struct NameExpr {
        pub name: String
    }

    #[derive(Debug)]
    pub struct NumberLitExpr {
        pub number: String
    }

    #[derive(Debug)]
    pub struct BinaryExpr {
        pub op: BinaryOp,
        pub left: Box<Expr>,
        pub right: Box<Expr>
    }

    #[derive(Debug)]
    pub enum BinaryOp {
        Add, Mul, Div, Sub
    }

    #[derive(Debug)]
    pub struct ReturnStmt {
        pub expr: Option<Expr>
    } 

    #[derive(Debug)]
    pub enum Code {
        Function(Function),
        ReturnStmt(ReturnStmt)
    }
}

#[derive(Debug)]
enum TokenKind {
    FunctionKeyword, ReturnKeyword,
    Ident(String),
    Char(char),
    Number(String),
    OpenCurly, CloseCurly, OpenParen, CloseParen,
    Semi, Comma, Add, Mul, Div, Sub,
    Whitespace
}

impl syntax::TokenKind for TokenKind {
    fn is_whitespace(&self) -> bool {
        matches!(self, TokenKind::Whitespace)
    }
}

type Token = syntax::Token<TokenKind>;
type TokenStream<'a> = syntax::TokenStream<'a, TokenKind>;

impl ast::Code {
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Code> {
        match stream.token().map(|x| x.kind()) {
            // Safe to unwrap here as the only way these can fail is if the initial keyword is not what is expected, which it is - because we wouldn't go into that parser otherwise
            Some(TokenKind::FunctionKeyword) => MatchResult::Ok(ast::Code::Function(syntax::parse!(stream, ast::Function::parse).unwrap())),
            Some(TokenKind::ReturnKeyword) => MatchResult::Ok(ast::Code::ReturnStmt(syntax::parse!(stream, ast::ReturnStmt::parse).unwrap())),
            
            _ => MatchResult::Fail
        }
    }
}

impl ast::Expr {
    fn parse_primary<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Expr> {
        match stream.token().map(|x| x.kind()) {
            Some(TokenKind::OpenParen) => {
                stream.step();
                let expr = Box::new(syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected expression inside of parenthesis")));
                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
                
                MatchResult::Ok(ast::Expr::Closed(ast::ClosedExpr { expr }))
            },
            Some(TokenKind::Number(s)) => {
                let number = s.to_string();
                stream.step();
                MatchResult::Ok(ast::Expr::NumberLit(ast::NumberLitExpr { number }))
            },
            Some(TokenKind::Ident(s)) => {
                let name = s.to_string();
                stream.step();
                MatchResult::Ok(ast::Expr::Name(ast::NameExpr { name }))
            },
            _ => MatchResult::Fail
        }
    }
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, ast::Expr::parse_primary));

        loop {
            match stream.token().map(|x| x.kind()) {
                Some(TokenKind::Add) | Some(TokenKind::Mul) | Some(TokenKind::Div) | Some(TokenKind::Sub) => {
                    let op = match stream.token().map(|x| x.kind()).unwrap() {
                        TokenKind::Add => ast::BinaryOp::Add,
                        TokenKind::Mul => ast::BinaryOp::Mul,
                        TokenKind::Sub => ast::BinaryOp::Sub,
                        TokenKind::Div => ast::BinaryOp::Div,
                        _ => unreachable!()
                    };
                    stream.step();

                    let right = syntax::ex!(syntax::parse!(stream, ast::Expr::parse), stream.error("Expected right hand side to expression"));

                    expr = ast::Expr::BinaryExpr(ast::BinaryExpr {
                        op,
                        left: Box::new(expr),
                        right: Box::new(right)
                    });
                },
                _ => break,
            }
        }

        syntax::MatchResult::Ok(expr)
    }
}

impl ast::ReturnStmt {
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::ReturnStmt> {
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::ReturnKeyword));

        let expr = syntax::parse!(stream, ast::Expr::parse);

        while syntax::tk_iss!(stream, TokenKind::Semi) {}

        syntax::MatchResult::Ok(ast::ReturnStmt {
            expr
        })
    }
}

impl ast::FunctionParam {
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::FunctionParam> {
        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident)).to_owned();
        stream.step();

        syntax::MatchResult::Ok(ast::FunctionParam {
            name
        })
    }
}

impl ast::Function {
    fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<ast::Function> {
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::FunctionKeyword));

        let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
        stream.step();

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenParen), stream.error("Expected '('"));

        let mut params = Vec::new();
        loop {
            params.push(match syntax::parse!(stream, ast::FunctionParam::parse) {
                Some(x) => x,
                None => break
            });

            while syntax::tk_iss!(stream, TokenKind::Comma) {}
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
        
        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::OpenCurly), stream.error("Expected '{'"));

        let mut code = Vec::new();
        loop {
            code.push(match syntax::parse!(stream, ast::Code::parse) {
                Some(x) => x,
                None => break
            });
        }

        syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseCurly), stream.error("Expected '}'"));

        syntax::MatchResult::Ok(ast::Function {
            name, params, code
        })
    }
}

struct Matcher;
impl syntax::TokenMatcher<TokenKind> for Matcher {
    fn next<'a>(&mut self, string: &'a str, offset: usize) -> Option<(usize, syntax::Token<TokenKind>)> {
        syntax::exact!(string, offset, 
            '{' => TokenKind::OpenCurly,
            '}' => TokenKind::CloseCurly,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semi,
            '+' => TokenKind::Add,
            '*' => TokenKind::Mul,
            '/' => TokenKind::Div,
            '-' => TokenKind::Sub
        );

        syntax::keywords!(string, offset,
            "function" => TokenKind::FunctionKeyword,
            "return" => TokenKind::ReturnKeyword
        );

        syntax::ident!(string, offset, TokenKind::Ident);
        syntax::whitespace!(string, offset, TokenKind::Whitespace);
        syntax::number!(string, offset, TokenKind::Number);
        
        syntax::char!(string, offset, TokenKind::Char);
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let path = args.get(1).expect(&format!("usage: {} <path/to/source.js>", args.get(0).unwrap()));

    let content = std::fs::read_to_string(path).expect(&format!("Could not open {}", path));

    let mut matcher = TokenStream::new(&content, Box::new(Matcher {}));
    matcher.step();

    match ast::Code::parse(&mut matcher) {
        syntax::MatchResult::Ok(code) => println!("{:#?}", code),
        syntax::MatchResult::Err(e) => println!("SyntaxError: {}-{}: {}", e.start(), e.end(), e.message()),
        syntax::MatchResult::Fail => print!("Could not match function"),
    };
}