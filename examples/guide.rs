//! A quick guide on the macro's syntax, showing it's major features.

#![allow(unused)]

fn main() {
    sylveon::parse! {
        // # Option definitions

        ignore?,    // optional value (Option<String>)
        all,        // switch (bool)
        verbose+,   // count (usize)

        // Option names will default to the variable's name, but can be given explicitly.
        force = f | force,

        // Options can also be documented with triple-slashes.

        /// Display version and exit
        version,

        // # Positional arguments

        // _ => todo!(),        // No arguments
        // path => todo!(),     // Single argument
        // ..paths => todo!(),  // Multiple arguments

        // Subcommands can be defined similarly. Note that positional arguments should come after
        // any subcommands.
        "check" | "c" => todo!(),

        // They can also have nested definitions, and even omit the body.
        "remove" | "rm" {
            dev,
            path => todo!(),
        }
    }
}

fn do_something(path: &str) {
    todo!()
}
