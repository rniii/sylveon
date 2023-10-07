// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

pub struct Args {
    unparsed: Vec<String>,
}

impl Args {
    pub fn get_unparsed(&mut self) -> &[String] {
        &self.unparsed
    }

    pub fn get_positional(&self) -> Option<&str> {
        let mut eoi = false;

        for i in &self.unparsed {
            if i == "--" {
                eoi = true;
                continue;
            }

            if eoi || !i.starts_with('-') {
                return Some(i);
            }
        }

        None
    }

    pub fn get_short_flag(&self, c: char) -> bool {
        for i in &self.unparsed {
            if i == "--" {
                break;
            }

            if i.starts_with('-') && !i.starts_with("--") {
                for x in i[2..].chars() {
                    if c == x {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn get_long_flag(&self, name: &'static str) -> bool {
        for i in &self.unparsed {
            if i == "--" {
                break;
            }

            if i.starts_with("--") && name == i[2..].split_once('=').map_or(&i[2..], |(f, _)| f) {
                return true;
            }
        }

        false
    }

    pub fn take_positional(&mut self) -> Option<String> {
        let mut eoi = false;

        for i in 0..self.unparsed.len() {
            if self.unparsed[i] == "--" {
                eoi = true;
                continue;
            }

            if eoi || !self.unparsed[i].starts_with('-') {
                return Some(self.unparsed.remove(i));
            }
        }

        None
    }

    pub fn take_long_param(&mut self, name: &'static str) -> Result<Option<String>, Error> {
        for i in 0..self.unparsed.len() {
            if self.unparsed[i] == "--" {
                break;
            }

            if self.unparsed[i].starts_with("--") && &self.unparsed[i][2..] == name {
                let flag = self.unparsed.remove(i);

                // value after equals --foo=bar
                if let Some((_, value)) = flag.split_once('=') {
                    return Ok(Some(value.to_owned()));
                }

                if self.unparsed.len() <= i {
                    return Err(Error::RequiredValue(Name::Long(name)));
                }

                return Ok(Some(self.unparsed.remove(i)));
            }
        }

        Ok(None)
    }

    pub fn take_short_param(&mut self, c: char) -> Result<Option<String>, Error> {
        for i in 0..self.unparsed.len() {
            if self.unparsed[i] == "--" {
                break;
            }

            if self.unparsed[i].starts_with('-') && self.unparsed[i].chars().nth(1) == Some(c) {
                let flag = self.unparsed.remove(i);

                // value after flag: -opath
                if flag.len() > 2 {
                    return Ok(Some(flag[2..].to_string()));
                }

                if self.unparsed.len() <= i {
                    return Err(Error::RequiredValue(Name::Short(c)));
                }
            }
        }

        Ok(None)
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            unparsed: std::env::args().skip(1).collect(),
        }
    }
}

#[derive(Debug)]
pub enum Name {
    Short(char),
    Long(&'static str),
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Short(c) => write!(f, "-{c}"),
            Self::Long(name) => write!(f, "--{name}"),
        }
    }
}

pub trait Parser {
    type Output;

    fn parse(args: &mut Args, names: &[Name]) -> Result<Self::Output, Error>;
}

impl Parser for Option<String> {
    type Output = Self;

    fn parse(args: &mut Args, names: &[Name]) -> Result<Self, Error> {
        for name in names {
            let v = match name {
                Name::Short(c) => args.take_short_param(*c)?,
                Name::Long(name) => args.take_long_param(name)?,
            };

            if v.is_some() {
                return Ok(v);
            }
        }

        Ok(None)
    }
}

impl Parser for bool {
    type Output = Self;

    fn parse(args: &mut Args, names: &[Name]) -> Result<Self::Output, Error> {
        for name in names {
            let v = match name {
                Name::Short(c) => args.get_short_flag(*c),
                Name::Long(name) => args.get_long_flag(name),
            };

            if v {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
