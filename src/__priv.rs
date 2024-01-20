// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

pub use crate::parser::{Opt, Parser};
pub use crate::{Args, Error, ErrorKind};

#[macro_export]
macro_rules! __init {
    ($args:ident; $opt:ident? $(, $($rest:tt)*)?) => {
        let mut $opt = None;
        $crate::__init! { $args; $($($rest)*)* }
    };
    ($args:ident; $opt:ident= $(, $($rest:tt)*)?) => {
        let mut $opt = None;
        $crate::__init! { $args; $($($rest)*)* }
    };
    ($args:ident; $opt:ident+ $(, $($rest:tt)*)?) => {
        let mut $opt = 0;
        $crate::__init! { $args; $($($rest)*)* }
    };
    ($args:ident; $opt:ident $(, $($rest:tt)*)?) => {
        let mut $opt = false;
        $crate::__init! { $args; $($($rest)*)* }
    };
    ($args:ident; $($rest:tt)*) => {};
}

#[macro_export]
macro_rules! __match {
    ($args:ident, $err:lifetime, $arg:ident; $opt:ident? $(, $($rest:tt)*)?) => {
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            match $args.value().and_then(|v| v.map_or(Err(Error::new(ErrorKind::MissingValue)), Ok)) {
                Ok(v) => $opt = Some(v.to_owned()),
                Err(e) => break $err Err(e),
            }

            continue;
        }

        $crate::__match! { $args, $err, $arg; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime, $arg:ident; $opt:ident= $(, $($rest:tt)*)?) => {
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            match $args.value().and_then(|v| v.map_or(Err(Error::new(ErrorKind::MissingValue)), Ok)) {
                Ok(v) => $opt = Some(v.to_owned()),
                Err(e) => break $err Err(e),
            }

            continue;
        }

        $crate::__match! { $args, $err, $arg; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime, $arg:ident; $opt:ident+ $(, $($rest:tt)*)?) => {
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            $opt += 1;

            continue;
        }

        $crate::__match! { $args, $err, $arg; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime, $arg:ident; $opt:ident $(, $($rest:tt)*)?) => {
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            $opt = true;

            continue;
        }

        $crate::__match! { $args, $err, $arg; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime, $arg:ident; $($rest:tt)+) => {
        let __val = match $arg {
            Some(Opt::Value(v)) => Some(v.to_owned()),
            Some(v) => break $err Err(Error::new(ErrorKind::Unexpected(v.to_string()))),
            None => None,
        };

        $crate::__cmd! { $args, $err, __val; $($rest)* }
    };
    ($args:ident, $err:lifetime, $arg:ident;) => {
        if let Some(Opt::Long("help") | Opt::Short('h')) = $arg {
            break $err Err(Error::new(ErrorKind::NeedsHelp));
        }

        if let Some(__arg) = $arg {
            break $err Err(Error::new(ErrorKind::Unexpected(__arg.to_string())));
        } else {
            break
        }
    };
}

#[macro_export]
macro_rules! __finish {
    ($args:ident, $err:lifetime; $opt:ident? $(, $($rest:tt)*)?) => {
        $crate::__finish! { $args, $err; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime; $opt:ident= $(, $($rest:tt)*)?) => {
        let $opt = match $opt {
            Some(v) => v,
            None => break $err Err(Error::MissingValue),
        };

        $crate::__finish! { $args, $err; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime; $opt:ident+ $(, $($rest:tt)*)?) => {
        $crate::__finish! { $args, $err; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime; $opt:ident $(, $($rest:tt)*)?) => {
        $crate::__finish! { $args, $err; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime; $($rest:tt)+) => {};
    ($args:ident, $err:lifetime;) => {
        Ok(())
    };
}

#[macro_export]
macro_rules! __cmd {
    ($args:ident, $err:lifetime, $arg:ident; $($cmd:tt)|+ $({ $($params:tt)* })? => $body:expr $(, $($rest:tt)*)?) => {
        if let $($crate::__pat_cmd!($cmd))|* = $arg.as_deref() {
            break $err $crate::try_parse! { $args; $($params)* }.map(|()| $body);
        }

        $crate::__cmd! { $args, $err, $arg; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime, $arg:ident; $($cmd:tt)|+ { $($params:tt)* } $(, $($rest:tt)*)?) => {
        if let $($crate::__pat_cmd!($cmd))|* = $arg.as_deref() {
            break $err $crate::try_parse! { $args; $($params)* }
        }

        $crate::__cmd! { $args, $err, $arg; $($($rest)*)* }
    };
    ($args:ident, $err:lifetime, $arg:ident;) => {
        break $err Err(Error::new($arg.map_or(ErrorKind::MissingCommand, ErrorKind::Unexpected)));
    };
}

#[macro_export]
macro_rules! __pat_cmd {
    ($str:literal) => {
        Some($str)
    };
    ($bind:ident) => {
        $bind
    };
    (_) => {
        None
    };
}
