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
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* $bind:ident => $body:expr $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        {
            let $bind = $arg;

            break match $args.next_opt() {
                Some(v) => Err(Error::Unexpected(v.to_string())),
                #[allow(unreachable_code)]
                None => Ok($body),
            }
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* ..$var:ident => $body:expr $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        {
            let mut $var = match $args.into_values() {
                Ok(v) => v,
                Err(opt) => break Err(Error::Unexpected(opt)),
            };

            if let Some(v) = $arg {
                $var.insert(0, v);
            }

            #[allow(unreachable_code)]
            {
                break Ok($body);
            }
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident; $(#[$($attr:tt)*])* $cmd:literal $(| $cmd2:literal)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $(#[$($attr)*])*
        if let Some($cmd $(| $cmd2)*) = $arg.as_deref() {
            $crate::__help! { $args, $(#[$($attr)*])* $cmd; $($($params)*)* }
            $crate::__init! { $($($params)*)* }

            match $crate::__loop! { $args; $($($params)*)* } {
                #[allow(unreachable_code)]
                Ok(v) => {
                    $(let () = v; break Ok($body);)*
                    break Ok(v);
                }
                Err(e) => break Err(e),
            }
        }

        $crate::__cmd! { $args, $arg; $($($rest)*)* }
    };
    ($args:ident, $arg:ident;) => {
        #[allow(unreachable_code)]
        {
            break Err($arg.map_or(Error::MissingCommand, Error::UnknownCommand));
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __help {
    ($args:ident; $($rest:tt)*) => {
        $args.context.name = ::std::env!("CARGO_PKG_NAME").into();
        $crate::__help_options! { [], [], [], $args; $($rest)* }
    };
    ($args:ident, $(#[$($attr:tt)*])* $str:literal; $($rest:tt)*) => {
        $args.context.name += concat!(" ", $str);
        $args.context.description = $crate::__doc! { $(#[$($attr)*])* };
        $crate::__help_options! { [], [], [], $args; $($rest)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __help_options {
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* (concat!("--", stringify!($opt), " <value>"), $crate::__doc! { $(#[$($attr)*])* })],
            [$($cmds),*],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident+ $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* (concat!("--", stringify!($opt)), $crate::__doc! { $(#[$($attr)*])* })],
            [$($cmds),*],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $opt:ident $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts,)* (concat!("--", stringify!($opt)), $crate::__doc! { $(#[$($attr)*])* })],
            [$($cmds),*],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $bind:ident => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($cmds),*],
            [$($usages,)* (concat!("[", stringify!($bind), "]"), $crate::__doc! { $(#[$($attr)*])* })],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* ..$var:ident => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($cmds),*],
            [$($usages),* (concat!("[", stringify!($var), "].."), $crate::__doc! { $(#[$($attr)*])* })],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident; $(#[$($attr:tt)*])* $str:literal $(| $str2:literal)* $({ $($params:tt)* })? $(=> $body:expr)? $(, $($rest:tt)*)?) => {
        $crate::__help_options! {
            [$($opts),*],
            [$($cmds,)* (concat!($str, $(", ", $str2)*), $crate::__doc! { $(#[$($attr)*])* })],
            [$($usages),*],
            $args; $($($rest)*)*
        }
    };
    ([$($opts:expr),*], [$($cmds:expr),*], [$($usages:expr),*], $args:ident;) => {
        $args.context.options = &[$($opts),*];
        $args.context.commands = &[$($cmds),*];
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
