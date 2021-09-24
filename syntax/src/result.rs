use crate::Span;

pub struct SyntaxError {
    span: Span,
    message: String
}

impl SyntaxError {
    pub fn new<T: Into<String>>(span: Span, message: T) -> SyntaxError {
        SyntaxError {
            span,
            message: message.into()
        }
    }

    pub fn start(&self) -> usize {
        self.span.start
    }

    pub fn end(&self) -> usize {
        self.span.end
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub enum MatchResult<T> {
    Ok(T),
    Err(SyntaxError),
    Fail
}