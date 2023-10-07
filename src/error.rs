// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

use crate::Name;

#[derive(Debug)]
pub enum Error {
    Missing,
    RequiredValue(Name),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "missing argument"),
            Self::RequiredValue(name) => write!(f, "option {name} requires a value"),
        }
    }
}

impl std::error::Error for Error {}
