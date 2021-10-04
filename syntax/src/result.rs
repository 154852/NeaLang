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
    /// Parsing successful, this is the value
    Ok(T),
    /// Pattern matches, but encountered an error
    Err(SyntaxError),
    /// Pattern does not match
    Fail
}