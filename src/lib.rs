// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

//! A very simple way to parse CLI arguments.

#![warn(missing_docs)]

#[doc(hidden)]
pub mod __priv;
mod parser;

pub use parser::{Args, Color, Opt, Style};

/// Parse CLI arguments.
///
/// If parsing fails or help is requested, this macro will exit the process. See [`try_parse`] for
/// manual handling.
///
/// See the [crate's documentation][crate] for more info.
#[macro_export]
macro_rules! parse {
    ($args:ident; $($rest:tt)*) => {{
        use $crate::__priv::*;

        $crate::__help! { $args; $($rest)* }

        $crate::__init! { $($rest)* }
        #[allow(unreachable_code, clippy::diverging_sub_expression)]
        match $crate::__loop! { $args; $($rest)* } {
            Ok(v) => v,
            Err(e) => e.terminate($args),
        }
    }};
    ($($rest:tt)*) => {{
        let mut __args = $crate::Args::new();
        $crate::parse! { __args; $($rest)* }
    }};
}

/// Parse CLI arguments.
///
/// This macro is similar to [`parse`], but does not exit on error, and returns a [`Result`]. If
/// you wish to exit with a custom error, see [`Error::terminate`].
///
/// See the [crate's documentation][crate] for more info.
#[macro_export]
macro_rules! try_parse {
    ($args:ident; $($rest:tt)*) => {{
        use $crate::__priv::*;

        $crate::__init! { $($rest)* }
        #[allow(unreachable_code, clippy::diverging_sub_expression)]
        $crate::__loop! { $args, $($rest)* }
    }};
    ($($rest:tt)*) => {{
        use $crate::__priv::*;

        let mut __args = $crate::Args::new();

        $crate::__init! { $($rest)* }
        #[allow(unreachable_code, clippy::diverging_sub_expression)]
        $crate::__loop! { __args; $($rest)* }
    }};
}

/// [`parse`] exit condition. This may occur with invalid arguments, or if `--help` is given,
/// [`Error::Help`].
#[derive(Debug)]
pub enum Error {
    /// The help message should be displayed
    Help,
    /// An argument was missing a flag
    MissingValue,
    /// Missing subcommand
    MissingCommand,
    /// Unexpected argument
    Unexpected(String),
    /// Unknown subcommand
    UnknownCommand(String),
    /// Missing required argument
    Required(String),
}

impl Error {
    /// Exit the program with this condition. [`Error::Help`] will display the help message and
    /// exit with code 0, otherwise display an error message and exit with code 1.
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
            Self::Required(v) => {
                args.style
                    .format_error(&format!("missing required argument: {v}"), &mut w)
                    .unwrap();
            }
        }

        std::process::exit(1)
    }
}
