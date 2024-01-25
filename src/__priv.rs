// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

pub use crate::{Args, Error, Opt};
pub use sylveon_macros::opt as __opt;

#[doc(hidden)]
#[macro_export]
macro_rules! __init {
    ($(#[$attr:meta])* $opt:ident? $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $opt = None;
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* $opt:ident+ $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $opt = 0;
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* $opt:ident $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $opt = false;
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* $bind:ident => $body:expr $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        let mut $bind = None;
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* ..$var:ident => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* $cmd:literal $(| $cmd2:literal)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $crate::__init! { $($($rest)*)* }
    };
    ($(#[$attr:meta])* _ => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__init! { $($($rest)*)* }
    };
    () => {};
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
    ($args:ident, $arg:ident; $(#[$attr:meta])* $opt:ident? $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        if let $crate::__pat! { $opt $(= $($v)|*)* } = $arg {
            $opt = match $args.value() {
                Some(v) => Some(v),
                None => break Err(Error::MissingValue),
            };

            continue;
        }

        $crate::__match! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$attr:meta])* $opt:ident+ $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        if let $crate::__pat! { $opt $(= $($v)|*)* } = $arg {
            $opt += 1;

            continue;
        }

        $crate::__match! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$attr:meta])* $opt:ident $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $(#[$attr])*
        if let $crate::__pat! { $opt $(= $($v)|*)* } = $arg {
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
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* $bind:ident => $body:expr $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        {
            match $arg {
                Some(v) if $bind.is_none() => $bind = Some(v),
                Some(v) => break Err(Error::Unexpected(v)),
                None => break Ok($body),
            }
            continue;
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* ..$var:ident => $body:expr $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        {
            let $var = match $args.into_values() {
                Ok(mut var) => {
                    if let Some(v) = $arg {
                        var.insert(0, v);
                    }
                    var
                }
                Err(opt) => break Err(Error::Unexpected(opt)),
            };

            break Ok($body);
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* $cmd:literal $(| $cmd2:literal)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        if let Some($cmd $(| $cmd2)*) = $arg.as_deref() {
            $crate::__help! { $args, $(#[$($attr)*])* $cmd; $($($params)*)* }
            $crate::__init! { $($($params)*)* }

            match $crate::__loop! { $args; $($($params)*)* } {
                Ok(v) => {
                    $(let () = v; break Ok($body);)*
                    break Ok(v);
                }
                Err(e) => break Err(e),
            }
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* _ => $body:expr $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        match $arg {
            Some(v) => break Err(Error::Unexpected(v)),
            None => break Ok($body),
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident;) => {
        break Err($arg.map_or(Error::MissingCommand, Error::UnknownCommand));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __help {
    ($args:ident; $($rest:tt)*) => {
        $args.context.name = ::std::env!("CARGO_PKG_NAME").into();
        $crate::__help_options! { [], [], $args; $($rest)* }
    };
    ($args:ident, $(#[$($attr:tt)*])* $str:literal; $($rest:tt)*) => {
        $args.context.name += concat!(" ", $str);
        $args.context.description = $crate::__doc! { $(#[$($attr)*])* };
        $crate::__help_options! { [], [], $args; $($rest)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __help_options {
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident? $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* ($crate::__help_opt! { $opt $(= $($v)|*)* }, " <value>", $crate::__doc! { $(#[$($attr)*])* })],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident+ $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* ($crate::__help_opt! { $opt $(= $($v)|*)* }, "", $crate::__doc! { $(#[$($attr)*])* })],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident $(= $($v:ident)|+)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* ($crate::__help_opt! { $opt $(= $($v)|*)* }, "", $crate::__doc! { $(#[$($attr)*])* })],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $bind:ident => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($usages,)* (concat!("[", stringify!($bind), "]"), $crate::__doc! { $(#[$($attr)*])* })],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* ..$var:ident => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($usages,)* (concat!("[", stringify!($var), "].."), $crate::__doc! { $(#[$($attr)*])* })],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $str:literal $(| $str2:literal)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($usages,)* (concat!($str, $(", ", $str2)*), $crate::__doc! { $(#[$($attr)*])* })],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* _ => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($usages,)* ("", $crate::__doc! { $(#[$($attr)*])* })],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($usages:expr),*], $args:ident;) => {
        $args.context.options = &[$($opts),*];
        $args.context.usages = &[$($usages),*];
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

#[doc(hidden)]
#[macro_export]
macro_rules! __pat {
    ($opt:ident = $($v:ident)|+) => {
        $(Some(__opt!($v)))|*
    };
    ($opt:ident) => {
        Some(__opt!($opt))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __help_opt {
    ($opt:ident = $($v:ident)|*) => {
        &[$(__opt!($v)),*]
    };
    ($opt:ident) => {
        &[__opt!($opt)]
    };
}
