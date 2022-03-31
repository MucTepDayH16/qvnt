use std::path::PathBuf;

use clap::ArgMatches;
use qvnt::prelude::{Int, Ast};

pub fn from_cli(
    cli: &ArgMatches
) -> Result<Int, Box<dyn std::error::Error>> {
    match cli.value_of("input") {
        Some(input) => {
            let path = PathBuf::from(input);
            let ast = Ast::from_file(&path)?;
            Ok(Int::new(&ast)?)
        }
        None => Ok(Int::default()),
    }
}