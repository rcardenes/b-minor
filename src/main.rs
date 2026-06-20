#![feature(string_into_chars)]
use std::{
    fs,
    process::ExitCode,
};

use anyhow::Result;
use clap::Parser as ClapParser;

use b_minor::{
    scan::Scanner,
    syn::Parser,
    sem::Semantic,
};

#[derive(ClapParser, Debug)]
struct Cli {
    /// Path to the input file
    input_file: String,

    /// Path to the output file
    #[clap(short, default_value_t = String::from("out.s"))]
    output: String,
}

fn compile(args: Cli) -> Result<()> {
    let program_text = fs::read_to_string(args.input_file)?;

    let mut parser = Parser::new(Scanner::new(program_text.into_chars().peekable()));
    let ast = parser.parse_top();

    let mut sem_analysis: Semantic = parser.into();
    sem_analysis.resolve(&ast);

    Ok(())
}

fn main() -> ExitCode {
    match compile(Cli::parse()) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error);
            ExitCode::FAILURE
        }
    }
}
