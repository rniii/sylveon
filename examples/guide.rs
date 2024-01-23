fn main() {
    sylveon::parse! {
        /// Ignore files matching this pattern
        ignore?,    // argument (Option<String>)
        /// List hidden files
        all,        // switch (bool)
        /// Increase verbosity
        verbose+,   // count (usize)

        ..paths => {
        }
   };
}
