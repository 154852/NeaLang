#[macro_export]
macro_rules! tk_is {
    ( $stream:expr , $p:pat ) => {
        $stream.token().map_or(false, |x| matches!(x.kind(), $p))
    };
}

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

#[macro_export]
macro_rules! parse {
    ( $stream:expr , $e:expr ) => {
        match $e ($stream) {
            ::syntax::MatchResult::Ok(x) => Some(x),
            ::syntax::MatchResult::Err(e) => return ::syntax::MatchResult::Err(e),
            ::syntax::MatchResult::Fail => None
        }
    };
}

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