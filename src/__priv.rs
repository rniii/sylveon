// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

pub use crate::{Args, Error, Opt};

#[doc(hidden)]
#[macro_export]
macro_rules! __init {
    ($(#[$attr:meta])* $opt:ident? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $opt = None;
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* $opt:ident+ $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $opt = 0;
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* $opt:ident $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $opt = false;
        $crate::__init! { $($($rest)*)* }
    };
    ($($rest:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __loop {
    ($args:ident; $($rest:tt)*) => {
        loop {
            let __arg = $args.next_opt();
            $crate::__match! { $args, __arg; $($rest)* }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __match {
    ($args:ident, $arg:ident; $(#[$attr:meta])* $opt:ident? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            $opt = match $args.value() {
                Some(v) => Some(v),
                None => break Err(Error::MissingValue),
            };

            continue;
        }

        $crate::__match! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$attr:meta])* $opt:ident+ $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            $opt += 1;

            continue;
        }

        $crate::__match! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$attr:meta])* $opt:ident $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        if let Some(Opt::Long(stringify!($opt))) = $arg {
            $opt = true;

            continue;
        }

        $crate::__match! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $($rest:tt)+) => {
        if let Some(Opt::Long("help") | Opt::Short('h')) = $arg {
            break Err(Error::Help);
        }

        let __val = match $arg {
            Some(Opt::Value(v)) => Some(v.to_owned()),
            Some(v) => break Err(Error::Unexpected(v.to_string())),
            None => None,
        };

        $crate::__cmd! { $args, __val; $($rest)* }
    };
    ($args:ident, $arg:ident;) => {
        if let Some(Opt::Long("help") | Opt::Short('h')) = $arg {
            break Err(Error::Help);
        }

        break $arg.map_or(Ok(()), |v| Err(Error::Unexpected(v.to_string())));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cmd {
    ($args:ident, $arg:ident; $($cmd:tt)|+ $({ $($params:tt)* })? => $body:expr $(, $($rest:tt)*)?) => {
        if let $($crate::__pat_cmd!($cmd))|* = $arg.as_deref() {
            $crate::__help! { $args, $($cmd)|*; $($($params)*)* }
            $crate::__init! { $($($params)*)* }

            break match $crate::__loop! { $args; $($($params)*)* } {
                Ok(()) => Ok($body),
                Err(e) => Err(e),
            }
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $($cmd:tt)|+ { $($params:tt)* } $(, $($rest:tt)*)?) => {
        if let $($crate::__pat_cmd!($cmd))|* = $arg.as_deref() {
            $crate::__help! { $args, $($cmd)|*; $($params)* }
            $crate::__init! { $($params)* }

            break $crate::__loop! { $args; $($params)* }
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident;) => {
        break Err($arg.map_or(Error::MissingCommand, Error::UnknownCommand));
    };
}

#[doc(hidden)]
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

#[doc(hidden)]
#[macro_export]
macro_rules! __help {
    ($args:ident; $($rest:tt)*) => {
        $args.context.name = ::std::env!("CARGO_PKG_NAME").into();
        $crate::__help_options! { [], [], $args; $($rest)* }
    };
    ($args:ident, $str:literal $(| $_:tt)*; $($rest:tt)*) => {
        $args.context.name += concat!(" ", $str);
        $crate::__help_options! { [], [], $args; $($rest)* }
    };
    ($args:ident, $bind:ident $(| $_:tt)*; $($rest:tt)*) => {};
    ($args:ident, _ $(| $_:tt)*; $($rest:tt)*) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __help_options {
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* (concat!("--", stringify!($opt), " <value>"), $crate::__doc! { $(#[$($attr)*])* })],
            [$($cmds),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident+ $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* (concat!("--", stringify!($opt)), $crate::__doc! { $(#[$($attr)*])* })],
            [$($cmds),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* (concat!("--", stringify!($opt)), $crate::__doc! { $(#[$($attr)*])* })],
            [$($cmds),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident; $bind:ident $(| $cmd:tt)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! { 
            [$($opts,)* (concat!("[", stringify!($bind), "]"), "")],
            [$($cmds),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident; $str:literal $(| $cmd:tt)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($cmds,)* ($str, "")],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident; $($cmd:tt)|+ $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($cmds),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], $args:ident;) => {
        $args.context.options = &[$($opts),*];
        $args.context.commands = &[$($cmds),*];
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __doc {
    (#[doc = $doc:literal] $(#[$($attr:tt)*])*) => {
        $doc
    };
    (#[$($_:tt)*] $(#[$($attr:tt)*])*) => {
        $crate::__doc! { $(#[$($attr)*])* }
    };
    () => {
        ""
    };
}
