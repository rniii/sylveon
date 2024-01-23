// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

#[doc(hidden)]
pub mod __priv;
pub mod parser;

pub use parser::{Args, Opt};

#[macro_export]
macro_rules! parse {
    ($args:ident; $($rest:tt)*) => {
        use $crate::__priv::*;

        $crate::__help! { $args; $($rest)* }

        $crate::__init! { $($rest)* }
        match $crate::__loop! { $args; $($rest)* } {
            Ok(v) => v,
            Err(e) => e.terminate($args),
        }
    };
    ($($rest:tt)*) => {
        let mut __args = $crate::Args::new();
        $crate::parse! { __args; $($rest)* }
    };
}

#[macro_export]
macro_rules! try_parse {
    ($args:ident; $($rest:tt)*) => {{
        use $crate::__priv::*;

        $crate::__init! { $($rest)* }
        $crate::__loop! { $args, $($rest)* }
    }};
    ($($rest:tt)*) => {{
        use $crate::__priv::*;

        let mut __args = $crate::Args::new();

        $crate::__init! { $($rest)* }
        $crate::__loop! { __args; $($rest)* }
    }};
}

#[derive(Debug)]
pub enum Error {
    Help,
    MissingValue,
    MissingCommand,
    Unexpected(String),
    UnknownCommand(String),
}

impl Error {
    pub fn terminate(self, args: parser::Args) -> ! {
        let mut w = std::io::stderr().lock();

        match self {
            Self::Help => {
                args.style
                    .format_help(&args.context, &mut std::io::stdout().lock())
                    .unwrap();

                std::process::exit(0);
            }
            Self::MissingValue => {
                let opt = args.peek_back().unwrap();
                args.style
                    .format_error(&format!("option '{opt}' requires a value"), &mut w)
                    .unwrap();
            }
            Self::MissingCommand => {
                let name = args.context.name;
                args.style
                    .format_error(&format!("missing subcommand for {name}"), &mut w)
                    .unwrap();
            }
            Self::Unexpected(v) => {
                args.style
                    .format_error(&format!("unexpected argument: {v}"), &mut w)
                    .unwrap();
            }
            Self::UnknownCommand(v) => {
                args.style
                    .format_error(&format!("unknown command: {v}"), &mut w)
                    .unwrap();
            }
        }

        std::process::exit(1)
    }
}
