#![allow(unused)]

fn main() {
    sylveon::parse! {
        // defining arguments:

        /// Ignore files matching this pattern
        ignore?,    // argument with value (Option<String>)
        /// List hidden files
        all,        // switch (bool)
        /// Increase verbosity
        verbose+,   // count (usize)

        // defining subcommands:

        /// Delete files
        "rm" {
            // subcommand parameters can be nested like this (including more commands)
            recursive,

            // variadic arguments:
            ..path => {
                for p in path {
                    match verbose {
                        0 => {},
                        1 => println!("{p}"),
                        _ => println!("Removed {p}"),
                    }
                }
            }
        },

        // positional argument:

        /// List files
        path => {
            let path = path.unwrap_or(".".to_owned());
            match verbose {
                0 => println!("{path}"),
                _ => println!("{path} {path}"),
            }
        }
    };
}
