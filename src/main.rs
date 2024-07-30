mod lexer;
mod term_color;

use std::env;
use term_color::*;

// get the version number of the compiler.
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // print the compiler banner to the console.
    println!(
        "{}Version: {VERSION}\n",
        r#"
█░░ █░█ ▄▀█   █▀▀ █▀█ █▀▄▀█ █▀█ █ █░░ █▀▀ █▀█
█▄▄ █▄█ █▀█   █▄▄ █▄█ █░▀░█ █▀▀ █ █▄▄ ██▄ █▀▄
"#
    );

    println!("{}", colored("test", Color::Red))
}
