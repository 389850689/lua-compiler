mod lexer;
mod term_color;

use std::env::{self, args};
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

    if args().len() <= 1 {
        println!(
            "{}: no source file provided.\n",
            colored("error", Color::Red)
        );
        std::process::exit(-1);
    }

    // attempt to read the lua file's bytes.
    let code = match std::fs::read_to_string(args().collect::<Vec<_>>()[1].clone()) {
        Ok(v) => v,
        Err(e) => {
            println!("{}: {e}.\n", colored("error", Color::Red));
            std::process::exit(-1);
        }
    };

    warning!("test");

    // tokenize the user generated code.
    let tokens = lexer::Lexer::new(&code).tokenize();

    println!(
        "{}: finished compilation.\n",
        colored("success", Color::Green)
    );
}
