use std::{
    str::FromStr,
    fmt, path::PathBuf,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    MissingCollon,
    UnknownCommand(String),
    UnspecifiedPath,
    UnspecifiedInt,
    UnspecifiedTag,
    WrongTagName(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingCollon => unreachable!(),
            Error::UnknownCommand(s) => write!(f, "Unknown command: {s}"),
            Error::UnspecifiedPath => write!(f, "Path to load file must be specified"),
            Error::UnspecifiedInt => {
                write!(f, "Integer must be specified to loop over comands")
            }
            Error::UnspecifiedTag => write!(f, "Tag name as string must be specified"),
            Error::WrongTagName(s) => write!(f, "There's no tag {s:?}"),
        }
    }
}

impl std::error::Error for Error {}

pub const HELP: &str = "QVNT Interpreter CLI

USAGE:
    : [COMMANDS...]

COMMANDS:
    watch|loop|l N          Repeat following commands N time
    tags|t TAG              Create TAG with current state
    goto TAG                Swap current state to TAG's state
    exit|quit|q             Exit interpreter
    class|c                 Show state of classical registers
    polar                   Show state of quantum registers in polar form
    prob|p                  Show state of quantum registers in probability form
    ops|o                   Snow current quantum operations queue
    finish|go|g             Start modulating quantum computer
    reset|r                 Clear current state
    names|alias|a           Show aliases for quantum and classical bits
    load|file|qasm FILE     Load state from FILE according to QASM language script
    help|h|?                Show this reference
";

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Loop(usize),
    Tags,
    Tag(String),
    Goto(String),
    Go,
    Reset,
    Load(PathBuf),
    Class,
    Polar,
    Probs,
    Ops,
    Names,
    Help,
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Commands(pub Vec<Command>);

impl Commands {
    fn parse<'a, I: Iterator<Item=&'a str>>(mut source: I) -> Result<Self, Error> {
        let size_hint = source.size_hint();
        let mut cmds = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));

        while let Some(cmd) = source.next() {
            match cmd {
                "watch" | "loop" | "l" => {
                    let int = source.next().and_then(|int| int.parse().ok()).ok_or(Error::UnspecifiedInt)?;
                    cmds.push(Command::Loop(int));
                },
                "tags" => {
                    cmds.push(Command::Tags);
                },
                "tag" | "t" => {
                    let tag = source.next().ok_or(Error::UnspecifiedTag)?;
                    cmds.push(Command::Tag(tag.to_string()));
                },
                "goto" => {
                    let tag = source.next().ok_or(Error::UnspecifiedTag)?;
                    cmds.push(Command::Goto(tag.to_string()));
                },
                "exit" | "quit" | "q" => {
                    cmds.push(Command::Quit);
                    break;
                },
                "class" | "c" => {
                    cmds.push(Command::Class);
                },
                "polar" => {
                    cmds.push(Command::Polar);
                },
                "prob" | "p" => {
                    cmds.push(Command::Probs);
                },
                "ops" | "o" => {
                    cmds.push(Command::Ops);
                },
                "finish" | "go" | "g" => {
                    cmds.push(Command::Go);
                },
                "reset" | "r" => {
                    cmds.push(Command::Reset);
                },
                "names" | "alias" | "a" => {
                    cmds.push(Command::Names);
                },
                "load" | "file" | "qasm" => {
                    let path = source.next().and_then(|path| path.parse().ok()).ok_or(Error::UnspecifiedPath)?;
                    cmds.push(Command::Load(path));
                },
                "help" | "h" | "?" => {
                    cmds.push(Command::Help);
                }
                cmd => return Err(Error::UnknownCommand(cmd.to_string())),
            }
        }
        
        Ok(Self(cmds))
    }
}

impl FromStr for Commands {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Error> {
        if let Some((_, ':')) = source.char_indices().nth(0) {
            let source = source.split_at(1).1.split_ascii_whitespace();
            Commands::parse(source)
        } else {
            Err(Error::MissingCollon)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd() {
        assert_eq!(
            "a b c d".parse::<Commands>(),
            Err(Error::MissingCollon)
        );
        assert_eq!(
            ":a b c d".parse::<Commands>(),
            Err(Error::UnknownCommand("b".to_string()))
        );
    }

    #[test]
    fn print_help() {
        println!("{}", HELP);
    }
}