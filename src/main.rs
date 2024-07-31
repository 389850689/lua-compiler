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
        log_error!("no source file provided.\n");
        std::process::exit(-1);
    }

    // attempt to read the lua file's bytes.
    let code = std::fs::read_to_string(args().collect::<Vec<_>>()[1].clone()).unwrap_or_else(|e| {
        log_error!("{e}.\n");
        std::process::exit(-1);
    });

    // tokenize the user generated code.
    let tokens = lexer::Lexer::new(&code).tokenize().unwrap_or_else(|| {
        println!();
        std::process::exit(-1);
    });

    log_success!("finished tokenization: {tokens:#?}.\n");

    log_success!("finished compilation.\n");
}
