use std::path::PathBuf;

use clap::ArgMatches;
use qvnt::prelude::{Ast, Int};

pub fn from_cli<'a>(cli: &'a ArgMatches) -> Result<(Int, Option<String>), Box<dyn std::error::Error>> {
    match cli.value_of("input") {
        Some(input) => {
            let path = PathBuf::from(input);
            let ast = Ast::from_file(&path)?;
            
            let path_tag = crate::process::file_tag(&path);
            Ok((Int::new(&ast)?, Some(path_tag)))
        }
        None => Ok((Int::default(), None)),
    }
}
