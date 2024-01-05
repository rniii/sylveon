// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

mod args;
mod error;

pub use args::{Args, Name, Parser};
pub use error::Error;

#[macro_export]
macro_rules! help {
    ($args:ident, $cmd:literal) => {
        println!("{} {}", $cmd, $args.context);
    };
}

#[macro_export]
macro_rules! cli {
    ($args:ident, $($rest:tt)*) => {
        $crate::__cli_internal! { @main $args $($rest)* }
        if let Some(arg) = $args.take_positional() {
            let arg = arg.as_str();
            $crate::__cli_internal! { @comm $args arg $($rest)* }
        }

        todo!();
    };
    ($($rest:tt)*) => {
        let mut __args = $crate::Args::default();
        $crate::cli! { __args, $($rest)* };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __cli_internal {
    (@main $args:ident $(#[$($attr:tt)*])* $name:ident $params:tt => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__cli_internal! { @main $args $($($rest)*)* }
    };
    (@main $args:ident $(#[$($attr:tt)*])* $name:ident => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__cli_internal! { @main $args $($($rest)*)* }
    };
    (@main $args:ident $(#[$($attr:tt)*])* { $($params:tt)* } => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__cli_internal! { @parse $args $($params)* }
        if $args.get_unparsed().is_empty() {
            return $body;
        }
        $crate::__cli_internal! { @main $args $($($rest)*)* }
    };
    (@main $args:ident) => {};

    (@comm $args:ident $comm:ident $(#[$($attr:tt)*])* { $($params:tt)* } => $body:expr $(, $($rest:tt)*)?) => {
        $crate::__cli_internal! { @comm $args $comm $($($rest)*)* }
    };
    (@comm $args:ident $comm:ident $(#[$($attr:tt)*])* $name:ident $rest_args:ident => $body:expr $(, $($rest:tt)*)?) => {
        if let stringify!($name) = $comm {
            let $rest_args = $args;
            return $body;
        }
        $crate::__cli_internal! { @comm $args $comm $($($rest)*)* }
    };
    (@comm $args:ident $comm:ident $(#[$($attr:tt)*])* $name:ident { $($params:tt)* } => $body:expr $(, $($rest:tt)*)?) => {
        if let stringify!($name) = $comm {
            $crate::__cli_internal! { @parse $args $($params)* }
            return $body;
        }
        $crate::__cli_internal! { @comm $args $comm $($($rest)*)* }
    };
    (@comm $args:ident $comm:ident $(#[$($attr:tt)*])* $name:ident => $body:expr $(, $($rest:tt)*)?) => {
        if let stringify!($name) = $comm {
            return $body;
        }
        $crate::__cli_internal! { @comm $args $comm $($($rest)*)* }
    };
    (@comm $args:ident $comm:ident) => {};

    (@parse $args:ident $(#[$($attr:tt)*])* $field:ident: $type:ty $(, $($rest:tt)*)?) => {
        let $field: $type = <$type as $crate::Parser>::parse(
            &mut $args,
            $crate::__cli_internal! { @names #[$($($attr)*),*] $field default [] }
        ).unwrap();
        $crate::__cli_internal! { @parse $args $($($rest)*)* }
    };
    (@parse $args:ident ..$rest:ident) => {};
    (@parse $args:ident) => {};

    (@names #[] $field:ident default [$($name:expr)*]) => {
        &[$($name,)* $crate::Name::Long(stringify!($field))]
    };
    (@names #[] $field:ident nodefault [$($name:expr)*]) => {
        &[$($name,)*]
    };
    (@names #[alias = $v:literal $(, $($rest:tt)*)?] $field:ident $m:ident [$($name:expr)*]) => {
        $crate::__cli_internal! { @names #[$($($rest)*)*] $field $m [$($name)* $v.into()] }
    };
    (@names #[long = $v:literal $(, $($rest:tt)*)?] $field:ident $m:ident [$($name:expr)*]) => {
        $crate::__cli_internal! { @names #[$($($rest)*)*] $field nodefault [$($name)* $crate::Name::Long($v)] }
    };
    (@names #[short = $v:literal $(, $($rest:tt)*)?] $field:ident $m:ident [$($name:expr)*]) => {
        $crate::__cli_internal! { @names #[$($($rest)*)*] $field nodefault [$($name)* $crate::Name::Short($v)] }
    };
    (@names #[positional $(, $($rest:tt)*)?] $field:ident $m:ident [$($name:expr)*]) => {
        $crate::__cli_internal! { @names #[$($($rest)*)*] $field nodefault [$($name)* $crate::Name::Positional(stringify!($field))] }
    };
    (@names #[$_:tt $($rest:tt)*] $field:ident $m:ident [$($name:expr)*]) => {
        $crate::__cli_internal! { @names #[$($rest)*] $field $m [$($name)*] }
    };
}
