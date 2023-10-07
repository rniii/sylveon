// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

mod args;
mod error;

pub use args::{Args, Name, Parser};
pub use error::Error;

#[macro_export]
macro_rules! cli {
    // #[attr]
    // command { #[attr] param: type } => body
    ($($(#[$($attr:tt)*])? $($name:ident)? $({ $($(#[$($f_attr:tt)*])* $field:ident: $type:ty),* $(,)? $(, ..$rest:ident)? })? => $body:expr $(,)?)*) => {
        let mut args = $crate::Args::default();

        $crate::cli! { args, $($($name)* $({ $($(#[$($f_attr)*])* $field: $type,)* $(..$rest)* })* => $body)* }
    };

    ($args:ident, $($(#[$($attr:tt)*])? $($name:ident)? $({ $($(#[$($f_attr:tt)*])* $field:ident: $type:ty),* $(,)? $(, ..$rest:ident)? })? => $body:expr $(,)?)*) => {
        $($crate::cli! { @main, $args $($name)* $({ $($(#[$($f_attr)*])* $field: $type,)* $(..$rest)* })* => $body })*
        if let Some(arg) = $args.get_positional() {
            $($crate::cli! { @cmd, $args arg $($name)* $({ $($(#[$($f_attr)*])* $field: $type,)* $(..$rest)* })* => $body })*
        }

        panic!("unexpected argument");
    };

    // { #[attr] param: type } => body
    (@main, $args:ident { $($(#[$($attr:tt)*])* $field:ident: $type:ty,)* $(..$rest:ident)? } => $body:expr) => {
        $($crate::cli! { @parse, $args $(#[$($attr)*])* $field: $type })*
        if $args.get_unparsed().is_empty() {
            $crate::cli! { @rest, $args $($rest)* }
            return $body;
        }
    };
    (@main, $args:ident $name:ident $_:tt => $body:expr) => {};

    // command { #[attr] param: type } => body
    (@cmd, $args:ident $arg:ident $name:ident { $($(#[$($attr:tt)*])* $field:ident: $type:ty,)* $(..$rest:ident)? } => $body:expr) => {
        if let stringify!($name) = $arg {
            $($crate::cli! { @parse, $args $(#[$($attr)*])* $field: $type })*
            return $body;
        }
    };
    (@cmd, $args:ident $arg:ident $_:tt => $body:expr) => {};

    // #[attr] param: type
    (@parse, $args:ident $(#[$($attr:tt)*])* $field:ident: $type:ty) => {
        let $field = <$type as $crate::Parser>::parse(&mut $args, $crate::cli!(@names, [] [] #[$($($attr)*),*] $field))?;
    };

    (@names, [$($short:expr)*] [] #[] $field:ident) => {
        &[$($short,)* $crate::Name::Long(stringify!($field))]
    };
    (@names, [$($short:expr)*] [$($long:expr)*] #[] $field:ident) => {
        &[$($short,)* $($long),*]
    };
    (@names, [$($short:expr)*] [$($long:expr)*] #[short = $v:literal $(, $a:ident = $b:literal)*] $field:ident) => {
        $crate::cli!(@names, [$($short)* $crate::Name::Short($v)] [$($long)*] #[$($a = $b),*] $field)
    };
    (@names, [$($short:expr)*] [$($long:expr)*] #[long = $v:literal $(, $a:ident = $b:literal)*] $field:ident) => {
        $crate::cli!(@names, [$($short)*] [$($long)* $crate::Name::Long($v)] #[$($a = $b),*] $field)
    };
    (@names, [$($short:expr)*] [$($long:expr)*] #[$($_:tt)* $(, $($rest:tt)*)?] $field:ident) => {
        $crate::cli!(@names, [$($short)*] [$($long)*] #[$($rest)*] $field)
    };

    (@rest, $args:ident $rest:ident) => {
        let $rest = $args;
    };
    (@rest, $args:ident) => {};
}
