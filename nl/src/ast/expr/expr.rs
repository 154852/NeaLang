use syntax::Span;

use crate::lexer::{TokenKind, TokenStream};
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenErrorKind, IrGenFunctionContext};
use crate::ast::TypeExpr;

use super::*;

#[derive(Debug)]
pub enum Expr {
    BinaryExpr(BinaryExpr),
    Name(NameExpr),
    Closed(ClosedExpr),
    NumberLit(NumberLitExpr),
    Call(CallExpr),
    MemberAccess(MemberAccessExpr),
    Index(IndexExpr),
    As(AsExpr),
    StringLit(StringLitExpr),
    SliceLit(SliceLitExpr),
    NewExpr(NewExpr),
    BoolLit(BoolLitExpr),
    Unary(UnaryExpr)
}

impl Expr {
    // This is quite unpleasant - After compilation the matches are fast (simple jump tables, so effectively equivalent to a vtable in speed),
    // but they are ugly to write regardless - unfortuantely this is the cost of the high level of safety rust provides, since using traits
    // and built in iheritence would never allow any upcasts

    pub fn span(&self) -> &Span {
        match self {
            Expr::BinaryExpr(be) => &be.span,
            Expr::Name(name) => &name.span,
            Expr::Closed(closed) => &closed.span,
            Expr::NumberLit(num) => &num.span,
            Expr::Call(call) => &call.span,
            Expr::MemberAccess(mem_acc) => &mem_acc.span,
            Expr::Index(index) => &index.span,
            Expr::As(a) => &a.span,
            Expr::StringLit(str) => &str.span,
            Expr::NewExpr(expr) => &expr.span,
            Expr::SliceLit(expr) => &expr.span,
            Expr::BoolLit(expr) => &expr.span,
            Expr::Unary(expr) => &expr.span
        }
    }

    /// Build the IR that references this value
    pub fn append_ir_value<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        match self {
            Expr::BinaryExpr(binary_expr) => binary_expr.append_ir(ctx, target, preferred),
            Expr::Name(name_expr) => name_expr.append_ir_value(ctx, target, preferred),
            Expr::Closed(closed_expr) => closed_expr.expr.append_ir_value(ctx, target, preferred),
            Expr::NumberLit(number_lit) => number_lit.append_ir(ctx, target, preferred),
            Expr::Call(call_expr) => call_expr.append_ir_in_expr(ctx, target, preferred),
            Expr::MemberAccess(member_access) => member_access.append_ir_value(ctx, target, preferred),
            Expr::Index(index_expr) => index_expr.append_ir_value(ctx, target, preferred),
            Expr::As(as_expr) => as_expr.append_ir(ctx, target, preferred),
            Expr::StringLit(string_expr) => string_expr.append_ir_value(ctx, target, preferred),
            Expr::NewExpr(new_expr) => new_expr.append_ir_value(ctx, target, preferred),
            Expr::SliceLit(slice_lit_expr) => slice_lit_expr.append_ir_value(ctx, target, preferred),
            Expr::BoolLit(bool_lit_expr) => bool_lit_expr.append_ir_value(ctx, target, preferred),
            Expr::Unary(unary) => unary.append_ir(ctx, target, preferred)
        }
    }

    /// Predicts the type of the object without building an IR - should always match the result of append_ir_value.
    /// This *DOES NOT* necessarily verify that the code is semantically correct, append_ir_value and construct_path do that
    pub fn resultant_type<'a>(&'a self, ctx: &IrGenFunctionContext<'a>, preferred: Option<&ir::ValueType>) -> Result<ir::ValueType, IrGenError> {
        match self {
            Expr::BinaryExpr(binary_expr) => binary_expr.resultant_type(ctx, preferred),
            Expr::Name(name_expr) => name_expr.resultant_type(ctx, preferred),
            Expr::Closed(closed_expr) => closed_expr.expr.resultant_type(ctx, preferred),
            Expr::NumberLit(number_lit) => number_lit.resultant_type(ctx, preferred),
            Expr::Call(call_expr) => call_expr.resultant_type(ctx, preferred),
            Expr::MemberAccess(member_access) => member_access.resultant_type(ctx, preferred),
            Expr::Index(index_expr) => index_expr.resultant_type(ctx, preferred),
            Expr::As(as_expr) => as_expr.resultant_type(ctx, preferred),
            Expr::StringLit(string_expr) => string_expr.resultant_type(ctx, preferred),
            Expr::NewExpr(new_expr) => new_expr.resultant_type(ctx, preferred),
            Expr::SliceLit(slice_lit_expr) => slice_lit_expr.resultant_type(ctx, preferred),
            Expr::BoolLit(bool_lit_expr) => bool_lit_expr.resultant_type(ctx, preferred),
            Expr::Unary(unary) => unary.resultant_type(ctx, preferred)
        }
    }

    /// Builds a path to the location that the expression returns to - allows it to be written to.
    pub fn construct_path_to<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget, preferred: Option<&ir::ValueType>) -> Result<(ir::StorableType, ir::ValuePath), IrGenError> {
        match self {
            Expr::BinaryExpr(binary_expr) => return Err(IrGenError::new(binary_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::Name(name_expr) => name_expr.construct_path_to(ctx, target, preferred),
            Expr::Closed(closed_expr) => closed_expr.expr.construct_path_to(ctx, target, preferred),
            Expr::NumberLit(number_lit) => return Err(IrGenError::new(number_lit.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::Call(call_expr) => return Err(IrGenError::new(call_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::MemberAccess(member_access) => member_access.construct_path_to(ctx, target, preferred),
            Expr::Index(index_expr) => index_expr.construct_path_to(ctx, target, preferred),
            Expr::As(as_expr) => return Err(IrGenError::new(as_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::StringLit(string_expr) => return Err(IrGenError::new(string_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::NewExpr(new_expr) => return Err(IrGenError::new(new_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::SliceLit(slice_lit_expr) => return Err(IrGenError::new(slice_lit_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::BoolLit(bool_lit_expr) => return Err(IrGenError::new(bool_lit_expr.span.clone(), IrGenErrorKind::InvalidLHS)),
            Expr::Unary(unary) => return Err(IrGenError::new(unary.span.clone(), IrGenErrorKind::InvalidLHS)),
        }
    }

    /// Will convert an expression to a Value of a given ValueType (if possible)
    pub fn as_value(&self, ir_unit: &ir::TranslationUnit, value_type: &ir::ValueType) -> Result<ir::Value, IrGenError> {
        match self {
            Expr::NumberLit(num) =>
                Ok(match value_type {
                    ir::ValueType::U8 => ir::Value::U8(num.number.parse().unwrap()),
                    ir::ValueType::I8 => ir::Value::I8(num.number.parse().unwrap()),
                    ir::ValueType::U16 => ir::Value::U16(num.number.parse().unwrap()),
                    ir::ValueType::I16 => ir::Value::I16(num.number.parse().unwrap()),
                    ir::ValueType::U32 => ir::Value::U32(num.number.parse().unwrap()),
                    ir::ValueType::I32 => ir::Value::I32(num.number.parse().unwrap()),
                    ir::ValueType::U64 => ir::Value::U64(num.number.parse().unwrap()),
                    ir::ValueType::I64 => ir::Value::I64(num.number.parse().unwrap()),
                    _ => ir::Value::I32(num.number.parse().unwrap()),
                }),
            Expr::As(as_expr) => {
                if as_expr.new_type.to_ir_value_type(ir_unit)? != *value_type {
                    return Err(IrGenError::new(as_expr.span.clone(), IrGenErrorKind::AssignmentTypeMismatch));
                }

                return as_expr.expr.as_value(ir_unit, value_type);
            },
            // FIXME we don't *technically* know this is a slice
            // TODO handle compile time known expressions, e.g. 1 + 2
            _ => Err(IrGenError::new(self.span().clone(), IrGenErrorKind::NonConstExprInSlice))
        }
    }

    // Operand precedence is implemented in NL via the parser:
    //      1. Primaries have the greatest precedence, these includes ClosedExprs, literals, member access CallExpr and so on
    //      2. Then * and / are parsed
    //      3. Followed by + and -
    //      4. Then >, >=, <, <=
    //      5. Finally && and ||
    // Doing this means that
    //      5*4 - 3/2 > 3 || a <= b + c
    // is equivalent to
    //      (((5*4) - (3/2)) > 3) || (a <= (b + c))
    // as expected

    fn parse_primary<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Expr> {
        let start = stream.tell_start();

        // 1. Match a single (non-repeating) expression
        let mut expr = match stream.token_kind() {
            Some(TokenKind::OpenParen) => {
                stream.step();
                let expr = Box::new(syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected expression inside of parenthesis")));
                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));
                
                Expr::Closed(ClosedExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    expr
                })
            },
            Some(TokenKind::OpenBracket) => {
                stream.step();
                
                let mut values = Vec::new();
                loop {
                    values.push(match syntax::parse!(stream, Expr::parse) {
                        Some(x) => x,
                        None => break
                    });
        
                    if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
                }
                
                syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseBracket), stream.error("Expected ']'"));

                Expr::SliceLit(SliceLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    values
                })
            },
            Some(TokenKind::Sub) => {
                stream.step();
                
                let right = Box::new(syntax::ex!(syntax::parse!(stream, Expr::parse_primary), stream.error("Expected expression after unary op")));
                
                Expr::Unary(UnaryExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    right,
                    op: UnaryOp::Neg
                })
            },
            Some(TokenKind::Number(s)) => {
                let number = s.to_string();
                stream.step();
                Expr::NumberLit(NumberLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    number
                })
            },
            Some(TokenKind::StringLit(s)) => {
                let s = s.to_string();
                stream.step();
                Expr::StringLit(StringLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    value: s
                })
            },
            Some(TokenKind::Ident(s)) => {
                let name = s.to_string();
                stream.step();
                Expr::Name(NameExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    name
                })
            },
            Some(TokenKind::SelfKeyword) => {
                stream.step();
                Expr::Name(NameExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    name: String::from("self")
                })
            },
            Some(TokenKind::TrueKeyword) => {
                stream.step();
                Expr::BoolLit(BoolLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    value: true
                })
            },
            Some(TokenKind::FalseKeyword) => {
                stream.step();
                Expr::BoolLit(BoolLitExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    value: false
                })
            },
            Some(TokenKind::NewKeyword) => {
                stream.step();
                
                let new_type = syntax::ex!(syntax::parse!(stream, TypeExpr::parse), stream.error("Expected a type"));
                
                Expr::NewExpr(NewExpr {
                    span: syntax::Span::new(start, stream.tell_start()),
                    new_type
                })
            },
            _ => return syntax::MatchResult::Fail
        };

        // 2. Then match 0+ added expressions, such as function calls, indexing, member access or type conversions
        loop {
            match stream.token_kind() {
                Some(TokenKind::OpenParen) => {
                    stream.step();
                    
                    let mut args = Vec::new();
                    loop {
                        args.push(match syntax::parse!(stream, Expr::parse) {
                            Some(x) => x,
                            None => break
                        });
            
                        if !syntax::tk_iss!(stream, TokenKind::Comma) { break }
                    }
    
                    syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseParen), stream.error("Expected ')'"));

                    expr = Expr::Call(CallExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
                        object: Box::new(expr),
                        args
                    });
                },
                Some(TokenKind::OpenBracket) => {
                    stream.step();
                    
                    let arg = syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected expression"));
    
                    syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::CloseBracket), stream.error("Expected ']'"));

                    expr = Expr::Index(IndexExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
                        object: Box::new(expr),
                        arg: Box::new(arg)
                    });
                },
                Some(TokenKind::Dot) => {
                    stream.step();

                    let name = syntax::ex!(syntax::tk_v!(stream, TokenKind::Ident), stream.error("Expected a name")).to_owned();
                    stream.step();

                    expr = Expr::MemberAccess(MemberAccessExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
                        object: Box::new(expr),
                        prop: name
                    });
                },
                Some(TokenKind::AsKeyword) => {
                    stream.step();

                    let new_type = syntax::ex!(syntax::parse!(stream, TypeExpr::parse), stream.error("Expected a type"));

                    expr = Expr::As(AsExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
                        expr: Box::new(expr), new_type
                    });
                },
                _ => break
            }
        }

        syntax::MatchResult::Ok(expr)
    }

    fn parse_op_mul_div<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, Expr::parse_primary));
        let start = stream.tell_start();

        loop {
            match stream.token_kind() {
                Some(TokenKind::Mul) | Some(TokenKind::Div) => {
                    let op = match stream.token_kind().unwrap() {
                        TokenKind::Mul => BinaryOp::Mul,
                        TokenKind::Div => BinaryOp::Div,
                        _ => unreachable!()
                    };
                    stream.step();

                    let right = syntax::ex!(syntax::parse!(stream, Expr::parse_primary), stream.error("Expected right hand side to expression"));

                    expr = Expr::BinaryExpr(BinaryExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
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

    fn parse_op_add_sub<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, Expr::parse_op_mul_div));
        let start = stream.tell_start();

        loop {
            match stream.token_kind() {
                Some(TokenKind::Add) | Some(TokenKind::Sub) => {
                    let op = match stream.token_kind().unwrap() {
                        TokenKind::Add => BinaryOp::Add,
                        TokenKind::Sub => BinaryOp::Sub,
                        _ => unreachable!()
                    };
                    stream.step();

                    let right = syntax::ex!(syntax::parse!(stream, Expr::parse_op_mul_div), stream.error("Expected right hand side to expression"));

                    expr = Expr::BinaryExpr(BinaryExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
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

    fn parse_op_cmp<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, Expr::parse_op_add_sub));
        let start = stream.tell_start();

        loop {
            match stream.token_kind() {
                Some(TokenKind::DblEq) | Some(TokenKind::NotEq) | 
                Some(TokenKind::Lt) | Some(TokenKind::Le) | Some(TokenKind::Gt) | Some(TokenKind::Ge) => {
                    let op = match stream.token_kind().unwrap() {
                        TokenKind::DblEq => BinaryOp::Eq,
                        TokenKind::NotEq => BinaryOp::Ne,
                        
                        TokenKind::Lt => BinaryOp::Lt,
                        TokenKind::Le => BinaryOp::Le,
                        TokenKind::Gt => BinaryOp::Gt,
                        TokenKind::Ge => BinaryOp::Ge,
                        _ => unreachable!()
                    };
                    stream.step();

                    let right = syntax::ex!(syntax::parse!(stream, Expr::parse_op_add_sub), stream.error("Expected right hand side to expression"));

                    expr = Expr::BinaryExpr(BinaryExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
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

    fn parse_op_bool<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Expr> {
        let mut expr = syntax::ex!(syntax::parse!(stream, Expr::parse_op_cmp));
        let start = stream.tell_start();

        loop {
            match stream.token_kind() {
                Some(TokenKind::BoolAnd) | Some(TokenKind::BoolOr) => {
                    let op = match stream.token_kind().unwrap() {
                        TokenKind::BoolAnd => BinaryOp::BoolAnd,
                        TokenKind::BoolOr => BinaryOp::BoolOr,
                        _ => unreachable!()
                    };
                    stream.step();

                    let right = syntax::ex!(syntax::parse!(stream, Expr::parse_op_cmp), stream.error("Expected right hand side to expression"));

                    expr = Expr::BinaryExpr(BinaryExpr {
                        span: syntax::Span::new(start, stream.tell_start()),
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

    pub fn parse<'a>(stream: &mut TokenStream<'a>) -> syntax::MatchResult<Expr> {
        Expr::parse_op_bool(stream)
    }
}