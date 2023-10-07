use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    sylveon::cli! {
        /// :3
        { #[short = 'f'] file: Option<String>, ..args } => {
            println!("file: {file:?}");

            Ok(())
        }
    }
}
