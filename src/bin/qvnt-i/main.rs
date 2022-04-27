mod cli;
mod int_tree;
mod lines;
mod process;
mod program;
mod utils;

fn main() -> Result<(), ()> {
    let cli = cli::CliArgs::new();
    program::Program::new(cli)?.run()?;
    Ok(())
}
