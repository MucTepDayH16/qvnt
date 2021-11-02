fn loop_fn(int: &mut Int, int_stack: &mut Vec<Int>, line: &String) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    match line.chars().next() {
        Some(':') => {
            let line = line[1..].split_whitespace();
			cmd::process(int, int_stack, line)?;
        },
        _ => {
            let line = "OPENQASM 2.0; ".to_string() + &line;
            let ast = Ast::from_source(line)?;
            int.add(&ast)?;
        },
    }

    Ok(())
}

mod cmd {
	use std::fmt;
	use super::*;
	
	#[derive(Debug, Clone, PartialEq)]
	enum Error {
		UnknownComand(String),
		UnspecifiedPath,
		EmptyStack,
		UnspecifiedInt,
	}
	
	impl fmt::Display for Error {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match self {
				Error::UnknownComand(s) =>
					write!(f, "unknown command: {}", s),
				Error::UnspecifiedPath =>
					write!(f, "you must specify path to load file"),
				Error::EmptyStack =>
					write!(f, "nothing to pop"),
				Error::UnspecifiedInt =>
					write!(f, "you must specify an integer to loop over comands"),
			}
		}
	}
	
	impl std::error::Error for Error {}
	
	pub fn process<'a, I>(int: &mut Int, int_stack: &mut Vec<Int>, mut line: I) -> Result<(), Box<dyn std::error::Error>>
	where I: Iterator<Item=&'a str> + Clone {
		while let Some(cmd) = line.next() {
			match cmd {
				"loop" =>
					match line.next().and_then(|s| s.parse::<usize>().ok()) {
						Some(num) =>
							for _ in 0..num {
								process(int, int_stack, line.clone())?;
							},
						None => Err(Error::UnspecifiedInt)?,
					},
				"push" =>
					int_stack.push(int.clone()),
				"pop" =>
					match int_stack.pop() {
						Some(popped) => *int = popped,
						None => Err(Error::EmptyStack)?,
					}
				"exit" | "quit" | "q" =>
					std::process::exit(0),
				"class" =>
					println!("{}\n", int.get_class().get_value(!0usize)),
				"polar" =>
					println!("{:.4?}\n", int.get_polar_wavefunction()),
				"prob" =>
					println!("{:.4?}\n", int.get_probabilities()),
				"ops" =>
					println!("{}\n", int.get_ops_tree()),
				"finish" | "f" => {
					int.reset().finish();
				},
				"reset" | "r" => {
					*int = Int::default();
				}
				"alias" | "a" | "names" => {
					println!("QREGs {}\nCREGs {}\n", int.get_q_alias(), int.get_c_alias())
				}
				"load" | "l" => {
					match line.next() {
						Some(path) => {
							let path = std::path::PathBuf::from(path);
							let ast = Ast::from_file(&path)?;
							let new = Int::new(&ast)?;
							let _ = std::mem::replace(int, new);
						},
						None => Err(Error::UnspecifiedPath)?,
					}
				}
				_ => Err(Error::UnknownComand(cmd.to_string()))?,
			}
		}
		
		Ok(())
	}
}