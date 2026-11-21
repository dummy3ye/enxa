//! Binary entry point for exa. All CLI work lives in `cli`.
//!
//! Owns: calling `cli::run()` and returning its process status.
//! Must NOT: contain solver logic or cube conventions.

mod cli;

fn main() {
    cli::run();
}
