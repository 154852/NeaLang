/// Evaluates to true if the current token exists and it's kind matches the given pattern
#[macro_export]
macro_rules! tk_is {
    ( $stream:expr , $p:pat ) => {
        $stream.token().map_or(false, |x| matches!(x.kind(), $p))
    };
}

/// Does the same as tk_is, but also steps if true.
#[macro_export]
macro_rules! tk_iss {
    ( $stream:expr , $p:pat ) => {
        {
            let res = $stream.token().map_or(false, |x| matches!(x.kind(), $p));
            if res { $stream.step(); }
            res
        }
    };
}

/// Takes a single value out of the tokenstream, of the given type - falling back to None
#[macro_export]
macro_rules! tk_v {
    ( $stream:expr , $( $i:ident )::* ) => {
        match $stream.token() {
            Some(x) => match x.kind() {
                $( $i )::* (v) => Some(v),
                _ => None
            },
            None => None
        }
    };
}

/// Call a parser and propogate the error
#[macro_export]
macro_rules! parse {
    ( $stream:expr , $e:expr ) => {
        match $e ($stream) {
            ::syntax::MatchResult::Ok(x) => Some(x),
            ::syntax::MatchResult::Err(e) => return ::syntax::MatchResult::Err(e),
            ::syntax::MatchResult::Fail => None
        }
    };
    ( $stream:expr , $e:expr , $( $arg:expr ),+ ) => {
        match $e ($stream, $( $arg ),+) {
            ::syntax::MatchResult::Ok(x) => Some(x),
            ::syntax::MatchResult::Err(e) => return ::syntax::MatchResult::Err(e),
            ::syntax::MatchResult::Fail => None
        }
    };
}

/// Either fail or return an error if the given value is not Some
#[macro_export]
macro_rules! ex {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return ::syntax::MatchResult::Fail
        }
    };

    ( $e:expr , $err:expr ) => {
        match $e {
            Some(x) => x,
            None => return ::syntax::MatchResult::Err($err)
        }
    };
}

/// Checks out of line that $e is true, and either failing or returning an error if it is not
#[macro_export]
macro_rules! req {
    ( $e:expr ) => {
        if !$e {
            return ::syntax::MatchResult::Fail;
        }
    };
    ( $e:expr , $err:expr ) => {
        if !$e {
            return ::syntax::MatchResult::Err($err);
        }
    };
}

/// Does the same as req, but steps on sucess
#[macro_export]
macro_rules! reqs {
    ( $s:expr , $e:expr ) => {
        if !$e {
            return ::syntax::MatchResult::Fail;
        }

        $s.step();
    };
    ( $s:expr , $e:expr , $err:expr ) => {
        if !$e {
            return ::syntax::MatchResult::Err($err);
        }

        $s.step();
    };
}