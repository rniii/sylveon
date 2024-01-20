// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

#[doc(hidden)]
pub mod __priv;
pub mod parser;

pub use parser::Args;

#[macro_export]
macro_rules! parse {
    ($($rest:tt)*) => {
        $crate::try_parse! { $($rest)* }
            .unwrap_or_else(|err| err.terminate())
    };
}

#[macro_export]
macro_rules! try_parse {
    ($args:ident; $($rest:tt)*) => {
        '__err: {
            use $crate::__priv::*;

            $crate::__init! { $args; $($rest)* }

            #[allow(unreachable_code)]
            loop {
                let __arg = $args.next_opt();
                $crate::__match! { $args, '__err, __arg; $($rest)* }
            }

            $crate::__finish! { $args, '__err; $($rest)* }
        }
    };
    ($($rest:tt)*) => {{
        let mut __args = $crate::Args::default();

        $crate::try_parse! { __args; $($rest)* }
    }};
}

#[derive(Debug)]
pub enum ErrorKind {
    NeedsHelp,
    MissingValue,
    MissingCommand,
    Unexpected(String),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    context: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind, context: None }
    }

    pub fn context(mut self, ctx: String) -> Self {
        self.context = Some(ctx);
        self
    }

    pub fn terminate(self) -> ! {
        std::process::exit(match self.kind {
            ErrorKind::NeedsHelp => todo!(),
            v => {
                println!("{v:?}");
                1
            }
        })
    }
}
