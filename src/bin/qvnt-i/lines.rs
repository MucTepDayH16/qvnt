use std::{fmt, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    UnknownCommand(String),
    UnspecifiedPath,
    UnspecifiedInt,
    TagError(crate::int_tree::Error),
    ExistedTagName(String),
    TagIsParent(String),
    TagIsHead(String),
    WrongTagName(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownCommand(s) => write!(f, "Unknown command: {s}"),
            Error::UnspecifiedPath => write!(f, "Path to load file must be specified"),
            Error::UnspecifiedInt => write!(f, "Integer must be specified for loop"),
            Error::TagError(e) => write!(f, "Tag error: {e}"),
            Error::ExistedTagName(s) => write!(f, "Tag name {s:?} already exists"),
            Error::TagIsParent(s) => write!(f, "Tag {s:?} is parent and cannot be removed"),
            Error::TagIsHead(s) => write!(f, "Tag {s:?} is head and cannot be removed"),
            Error::WrongTagName(s) => write!(f, "There's no tag {s:?}"),
        }
    }
}

impl std::error::Error for Error {}

pub const HELP: &str = "QVNT Interpreter CLI

USAGE:
    : [COMMANDS...]

COMMANDS:
    loop|l N    Repeat following commands N time
    tag TAG     Create TAG with current state
    to TAG      Swap current state to TAG's state
    tags        Show the list of previously created tags
    quit|q      Exit interpreter
    class|c     Show state of classical registers
    polar       Show state of quantum registers in polar form
    prob|p      Show state of quantum registers in probability form
    ops|o       Snow current quantum operations queue
    go|g        Start modulating quantum computer
    reset|r     Clear current state
    names|n     Show aliases for quantum and classical bits
    load FILE   Load state from FILE according to QASM language script
    help|h|?    Show this reference
";

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Loop(usize),
    Tags(crate::int_tree::Command),
    Go,
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
pub enum Line {
    Commands(Vec<Command>),
    Qasm,
}

impl Line {
    fn parse_command<'a, I: Iterator<Item = &'a str>>(
        mut source: I,
    ) -> Result<Vec<Command>, Error> {
        let size_hint = source.size_hint();
        let mut cmds = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));

        while let Some(cmd) = source.next() {
            match cmd {
                "loop" | "l" => {
                    let int = source
                        .next()
                        .and_then(|int| int.parse().ok())
                        .ok_or(Error::UnspecifiedInt)?;
                    cmds.push(Command::Loop(int));
                }
                "tag" => {
                    cmds.push(Command::Tags(crate::int_tree::Command::parse_command(
                        &mut source,
                    )?));
                }
                "exit" | "quit" | "q" => {
                    cmds.push(Command::Quit);
                    break;
                }
                "class" | "c" => {
                    cmds.push(Command::Class);
                }
                "polar" => {
                    cmds.push(Command::Polar);
                }
                "prob" | "p" => {
                    cmds.push(Command::Probs);
                }
                "ops" | "o" => {
                    cmds.push(Command::Ops);
                }
                "go" | "g" => {
                    cmds.push(Command::Go);
                }
                "names" | "n" => {
                    cmds.push(Command::Names);
                }
                "load" | "file" | "qasm" => {
                    let path = source
                        .next()
                        .and_then(|path| path.parse().ok())
                        .ok_or(Error::UnspecifiedPath)?;
                    cmds.push(Command::Load(path));
                }
                "help" | "h" | "?" => {
                    cmds.push(Command::Help);
                }
                cmd => return Err(Error::UnknownCommand(cmd.to_string())),
            }
        }

        Ok(cmds)
    }
}

impl FromStr for Line {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Error> {
        if let Some((_, ':')) = source.char_indices().nth(0) {
            let source = source.split_at(1).1.split_ascii_whitespace();
            Line::parse_command(source).map(Line::Commands)
        } else {
            Ok(Line::Qasm)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd() {
        assert_eq!("a b c d".parse::<Line>(), Ok(Line::Qasm));
        assert_eq!(
            ":a b c d".parse::<Line>(),
            Err(Error::UnknownCommand("a".to_string()))
        );
    }

    #[test]
    fn print_help() {
        println!("{}", HELP);
    }
}
