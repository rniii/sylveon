// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, ErrorKind};

#[derive(Debug)]
pub enum Opt<'a> {
    Short(char),
    Long(&'a str),
    Value(&'a str),
}

impl std::fmt::Display for Opt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Short(c) => write!(f, "-{c}"),
            Self::Long(l) => write!(f, "--{l}"),
            Self::Value(v) => v.fmt(f),
        }
    }
}

pub trait Parser {
    fn next_opt(&mut self) -> Option<Opt>;

    fn value(&mut self) -> Result<Option<&str>, Error>;
}

enum State {
    Read(usize),
    Eoi(usize),
    Short(usize, usize),
    Empty,
}

pub struct Args {
    args: Vec<String>,
    state: State,
}

impl Args {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            state: State::Read(0),
        }
    }
}

impl Parser for Args {
    fn next_opt(&mut self) -> Option<Opt> {
        loop {
            match &mut self.state {
                State::Read(i) => {
                    let arg = match self.args.get(*i) {
                        Some(v) => v,
                        None => {
                            self.state = State::Empty;
                            continue;
                        }
                    };

                    *i += 1;

                    if let Some(short) = arg.strip_prefix('-') {
                        if let Some(long) = short.strip_prefix('-') {
                            if long.is_empty() {
                                self.state = State::Eoi(*i);
                                continue;
                            }

                            return Some(Opt::Long(long));
                        }

                        self.state = State::Short(*i - 1, 1);
                        continue;
                    }

                    return Some(Opt::Value(arg));
                }
                State::Short(i, j) => match self.args[*i][*j..].chars().next() {
                    Some(c) => {
                        *j += c.len_utf8();
                        return Some(Opt::Short(c));
                    }
                    None => {
                        self.state = State::Read(*i + 1);
                        continue;
                    }
                },
                State::Eoi(i) => return self.args.get(*i).map(|v| Opt::Value(v)),
                State::Empty => return None,
            }
        }
    }

    fn value(&mut self) -> Result<Option<&str>, Error> {
        match self.state {
            State::Short(i, j) => {
                self.state = State::Read(i + 1);
                Ok(Some(&self.args[i][j..]))
            }
            _ => Ok(match self.next_opt() {
                Some(Opt::Value(v)) => Some(v),
                Some(v) => return Err(Error::new(ErrorKind::Unexpected(v.to_string()))),
                None => None,
            }),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new(std::env::args().skip(1).collect())
    }
}
