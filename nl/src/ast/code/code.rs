use crate::ast::Expr;
use crate::irgen::{IrGenCodeTarget, IrGenError, IrGenFunctionContext};
use crate::lexer::{TokenKind, TokenStream};

use super::{Assignment, ForStmt, IfStmt, ReturnStmt, VarDeclaration, DropStmt};

#[derive(Debug)]
pub enum Code {
    ReturnStmt(ReturnStmt),
    VarDeclaration(VarDeclaration),
    ExprStmt(Expr),
    Assignment(Assignment),
    IfStmt(IfStmt),
    ForStmt(ForStmt),
    DropStmt(DropStmt)
}

impl Code {
    /// Parse a code statement. terminated is a flag indicating whether or not the line should end with a ';' - used by for loops
    pub fn parse<'a>(stream: &mut TokenStream<'a>, terminated: bool) -> syntax::MatchResult<Code> {
        if terminated {
            while syntax::tk_iss!(stream, TokenKind::Semi) {}
        }
        
        let start = stream.tell_start();
        let code = match stream.token_kind() {
            // Safe to unwrap here as the only way these can fail is if the initial keyword is not what is expected, which it is - because we wouldn't go into that parser otherwise
            Some(TokenKind::ReturnKeyword) => Code::ReturnStmt(syntax::parse!(stream, ReturnStmt::parse, terminated).unwrap()),
            Some(TokenKind::VarKeyword) => Code::VarDeclaration(syntax::parse!(stream, VarDeclaration::parse, terminated).unwrap()),
            Some(TokenKind::IfKeyword) => Code::IfStmt(syntax::parse!(stream, IfStmt::parse).unwrap()),
            Some(TokenKind::ForKeyword) => Code::ForStmt(syntax::parse!(stream, ForStmt::parse).unwrap()),
            Some(TokenKind::DropKeyword) => Code::DropStmt(syntax::parse!(stream, DropStmt::parse, terminated).unwrap()),
            
            // Special case for ExprStmt / Assignment
            _ => {
                // 1. Parse an expression
                let expr = syntax::ex!(syntax::parse!(stream, Expr::parse));

                // 2. If the next token is an equal, it is an assignment...
                if syntax::tk_iss!(stream, TokenKind::Eq) {
                    // Parse the RHS
                    let right = syntax::ex!(syntax::parse!(stream, Expr::parse), stream.error("Expected RHS"));
                    
                    if terminated { syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'")); }

                    Code::Assignment(Assignment {
                        span: syntax::Span::new(start, stream.tell_start()),
                        left: expr,
                        right
                    })
                } else {
                    // 3. ... otherwise it is just an expression statement
                    if terminated { syntax::reqs!(stream, syntax::tk_is!(stream, TokenKind::Semi), stream.error("Expected ';'")); }

                    Code::ExprStmt(expr)
                }
            }
        };

        if terminated {
            while syntax::tk_iss!(stream, TokenKind::Semi) {}
        }

        syntax::MatchResult::Ok(code)
    }

    pub fn append_ir<'a>(&'a self, ctx: &mut IrGenFunctionContext<'a>, target: &mut IrGenCodeTarget) -> Result<(), IrGenError> {
        match self {
            Code::ReturnStmt(stmt) => stmt.append_ir(ctx, target),
            Code::VarDeclaration(vardecl) => vardecl.append_ir(ctx, target),
            Code::ExprStmt(expr) => {
                match expr {
                    Expr::Call(call_expr) => call_expr.append_ir_out_expr(ctx, target)?,
                    _ => {
                        expr.append_ir_value(ctx, target, None)?;
                        target.push(ir::Ins::Drop); // Drop result as it's not used
                    }
                }

                Ok(())
            },
            Code::Assignment(assignment) => assignment.append_ir(ctx, target),
            Code::IfStmt(if_stmt) => if_stmt.append_ir(ctx, target),
            Code::ForStmt(for_stmt) => for_stmt.append_ir(ctx, target),
            Code::DropStmt(drop_stmt) => drop_stmt.append_ir(ctx, target)
        }
    }
}