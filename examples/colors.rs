//! Example using custom color codes for the command output.

use sylveon::{Color, Style};

fn main() {
    // Color can have any sgr escape, it is a shorthand for `\e[` ... `m`.
    // If the `NO_COLOR` environment variable is present, it will not be output at all.
    let mut args = sylveon::Args::new().style(Style {
        // bold combined with a 256-color escape
        primary: Color::new("1;38;5;182"),
        // reset then another color
        secondary: Color::new("0;38;5;224"),
        ..Style::default()
    });

    sylveon::parse! { args;
        /// Does something
        "test" => {}
    }
}
