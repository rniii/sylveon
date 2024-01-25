// sylveon, a tiny cli parser
// Copyright (c) 2024 rini
//
// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::io;

/// A command line option. This is used by [`parse`](crate::parse).
#[derive(Debug)]
pub enum Opt<'a> {
    /// Short flag (-c)
    Short(char),
    /// Long flag (--foo)
    Long(&'a str),
    /// A plain argument
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
    pub description: &'a str,
    pub usages: &'a [(&'a str, &'a str)],
    pub options: &'a [(&'a [Opt<'a>], &'a str, &'a str)],
    pub commands: &'a [(&'a str, &'a str)],
}

/// Argument parser. This can be given to [`parse`](crate::parse) to customise which arguments are
/// parsed, as well as the formatting.
pub struct Args {
    args: Vec<String>,
    state: State,
    pub(crate) style: Style,
    #[doc(hidden)]
    pub context: Context<'static>,
}

impl Args {
    /// Create an argument parser from command-line arguments.
    #[must_use]
    pub fn new() -> Self {
        Self::from(std::env::args().skip(1))
    }

    /// Create an argument parser from the given arguments.
    pub fn from(args: impl IntoIterator<Item = String>) -> Self {
        Self {
            args: args.into_iter().collect(),
            state: State::Read(0),
            style: Style::default(),
            context: Context::default(),
        }
    }

    /// Set this command's [`Style`], changing output colors.
    ///
    /// Color output can be disabled by setting the `NO_COLOR` environment variable, also via
    /// [`std::env::set_var`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Read the next option in the arguments. Used by [`parse`](crate::parse).
    ///
    /// Returns [None] if empty.
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

    /// Get the next value from the arguments. Used by [`parse`](crate::parse).
    ///
    /// Similar to `args.next_opt().map(|v| v.to_string())`, but also handles values in short
    /// arguments (`-ofile`).
    pub fn value(&mut self) -> Option<String> {
        match self.state {
            State::Short(i, j) => {
                self.state = State::Read(i + 1);
                Some(self.args[i][j..].to_owned())
            }
            _ => self.next_opt().map(|v| v.to_string()),
        }
    }

    /// Try to read all the remaining arguments as values. Used by [`parse`](crate::parse).
    ///
    /// # Errors
    ///
    /// If a non-[value](Opt::Value) argument is found, it will be returned as [`Err`].
    pub fn into_values(&mut self) -> Result<Vec<String>, String> {
        let mut values = Vec::new();

        loop {
            match self.next_opt() {
                Some(Opt::Value(v)) => values.push(v.to_owned()),
                Some(opt) => return Err(opt.to_string()),
                None => return Ok(values),
            }
        }
    }

    /// Peek the previous argument. Used by [`parse`](crate::parse) when formatting errors.
    pub fn peek_back(&self) -> Option<&str> {
        match self.state {
            State::Empty => self.args.last().map(String::as_str),
            _ => None,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

/// Colors for [`Error`](crate::Error) output.
pub struct Style {
    /// Primary color, used in headers
    pub primary: Color,
    /// Secondary color, used as an accent color
    pub secondary: Color,
    /// Tertiary color, used in longer sections of text
    pub tertiary: Color,
    /// Error color, in the error message's prefix
    pub error: Color,
}

impl Style {
    pub(crate) fn format_help(&self, ctx: &Context, f: &mut impl io::Write) -> io::Result<()> {
        let name = &ctx.name;
        let description = ctx.description.trim();

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

        if ctx.usages.is_empty() {
            writeln!(f, "{p}Usage: {s}{name}")?;
        } else {
            writeln!(f, "{p}Usage:")?;
            for (usage, doc) in ctx.usages {
                let cmd = format!("{name} {usage}");
                writeln!(f, "    {s}{cmd:<18}  {t}{}", doc.trim())?;
            }
        }

        if !description.is_empty() {
            writeln!(f, "\n{t}{description}")?;
        }

        if !ctx.options.is_empty() {
            writeln!(f, "\n{p}Options:{t}")?;
            for (opts, v, doc) in ctx.options {
                let opts = opts
                    .iter()
                    .map(Opt::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
                    + v;

                writeln!(f, "    {opts:<22}  {}", doc.trim())?;
            }
        }

        if !ctx.commands.is_empty() {
            writeln!(f, "\n{p}Commands:{t}")?;
            for (cmd, doc) in ctx.commands {
                writeln!(f, "    {cmd:<22}  {}", doc.trim())?;
            }
        }

        Ok(())
    }

    pub(crate) fn format_error(&self, error: &str, f: &mut impl io::Write) -> io::Result<()> {
        writeln!(f, "{}error: {}{error}", self.error, self.tertiary)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            primary: Color::new("93"),
            secondary: Color::new("96"),
            tertiary: Color::new(""),
            error: Color::new("31"),
        }
    }
}

/// An [ANSI formatting sequence], which can be disabled.
///
/// [ANSI formatting sequence]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
#[derive(Clone, Copy)]
pub struct Color(Option<&'static str>);

impl Color {
    /// Create a new color
    #[must_use]
    pub fn new(color: &'static str) -> Self {
        Self(Some(color))
    }

    /// Disable outputting this color.
    pub fn disable(&mut self) {
        self.0 = None;
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Color(Some(c)) = self {
            write!(f, "\x1b[{c}m")?;
        }

        Ok(())
    }
}
