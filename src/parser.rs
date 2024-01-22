// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::io;

#[derive(Debug)]
pub enum Opt<'a> {
    Short(char),
    Long(&'a str),
    Value(&'a str),
}

impl std::fmt::Display for Opt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Short(c) => write!(f, "-{c}"),
            Self::Long(l) => write!(f, "--{l}"),
            Self::Value(v) => v.fmt(f),
        }
    }
}

enum State {
    Read(usize),
    Eoi(usize),
    Short(usize, usize),
    Empty,
}

#[derive(Default)]
pub struct Context<'a> {
    pub name: Cow<'a, str>,
    pub options: &'a [(&'a str, &'a str)],
    pub commands: &'a [(&'a str, &'a str)],
}

pub struct Args {
    args: Vec<String>,
    state: State,
    pub style: Style,
    #[doc(hidden)]
    pub context: Context<'static>,
}

impl Args {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            state: State::Read(0),
            style: Style::default(),
            context: Context::default(),
        }
    }

    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn next_opt(&mut self) -> Option<Opt> {
        loop {
            match &mut self.state {
                State::Read(i) => match self.args.get(*i) {
                    Some(arg) => match arg.strip_prefix('-') {
                        Some(short) if !short.is_empty() => match short.strip_prefix('-') {
                            Some(long) if !long.is_empty() => {
                                *i += 1;
                                return Some(Opt::Long(long));
                            }
                            Some(_) => self.state = State::Eoi(*i + 1),
                            None => self.state = State::Short(*i, 1),
                        },
                        Some(_) => self.state = State::Eoi(*i + 1),
                        None => {
                            *i += 1;
                            return Some(Opt::Value(arg));
                        }
                    },
                    None => self.state = State::Empty,
                },
                State::Short(i, j) => match self.args[*i][*j..].chars().next() {
                    Some(c) => {
                        *j += c.len_utf8();
                        return Some(Opt::Short(c));
                    }
                    None => self.state = State::Read(*i + 1),
                },
                State::Eoi(i) => return self.args.get(*i).map(|v| Opt::Value(v)),
                State::Empty => return None,
            }
        }
    }

    pub fn value(&mut self) -> Option<String> {
        match self.state {
            State::Short(i, j) => {
                self.state = State::Read(i + 1);
                Some(self.args[i][j..].to_owned())
            }
            _ => self.next_opt().map(|v| v.to_string()),
        }
    }

    pub fn peek_back(&self) -> Option<&str> {
        match self.state {
            State::Empty => self.args.last().map(String::as_str),
            _ => None,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new(std::env::args().skip(1).collect())
    }
}

pub struct Style {
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub error: Color,
}

impl Style {
    pub fn format_help(&self, ctx: &Context, f: &mut impl io::Write) -> io::Result<()> {
        let &Style {
            primary: mut p,
            secondary: mut s,
            tertiary: mut t,
            ..
        } = self;

        if std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()) {
            p.disable();
            s.disable();
            t.disable();
        }

        writeln!(f, "{p}Usage: {s}{}", ctx.name)?;

        if !ctx.options.is_empty() {
            writeln!(f, "\n{p}Options:{t}")?;
            for (opt, desc) in ctx.options {
                writeln!(f, "    {opt:<12}  {desc}")?;
            }
        }

        if !ctx.commands.is_empty() {
            writeln!(f, "\n{p}Options:{t}")?;
            for (cmd, desc) in ctx.commands {
                writeln!(f, "    {cmd:<12}  {desc}")?;
            }
        }

        Ok(())
    }

    pub fn format_error(&self, error: &str, f: &mut impl io::Write) -> io::Result<()> {
        writeln!(f, "{}error: {}{error}", self.error, self.tertiary)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            primary: Color::new("93"),
            secondary: Color::new("96"),
            tertiary: Color::new("0"),
            error: Color::new("31"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Color(Option<&'static str>);

impl Color {
    pub fn new(color: &'static str) -> Self {
        Self(Some(color))
    }

    pub fn disable(&mut self) {
        self.0 = None;
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Color(Some(c)) = self {
            write!(f, "\x1b[{c}m")?;
        }

        Ok(())
    }
}
