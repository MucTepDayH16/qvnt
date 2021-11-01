fn loop_fn(int: &mut Int, int_stack: &mut Vec<Int>, line: &String) -> Result<(), Box<dyn std::error::Error>> {
    println!();
    match line.chars().next() {
        Some(':') => {
            let mut line = line[1..].split_whitespace();
            while let Some(cmd) = line.next() {
                match cmd {
                    "push" =>
                        int_stack.push(int.clone()),
                    "pop" =>
                        match int_stack.pop() {
                            Some(popped) => *int = popped,
                            None => eprintln!("nothing to pop\n"),
                        }
                    "exit" | "quit" | "q" =>
                        std::process::exit(0),
                    "class" =>
                        println!("{}\n", int.get_class().get_value(!0usize)),
                    "polar" =>
                        println!("{:.2?}\n", int.get_polar_wavefunction()),
                    "prob" =>
                        println!("{:.2?}\n", int.get_probabilities()),
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
                                std::mem::drop(std::mem::take(int));
                                *int = Int::new(&ast)?;
                            },
                            None => eprintln!("you must specify path to load!\n"),
                        }
                    }
                    _ => eprintln!("unknown command!\n"),
                }
            }
        },
        _ => {
            let line = "OPENQASM 2.0; ".to_string() + &line;
            let ast = Ast::from_source(line)?;
            int.add(&ast)?;
            //  int.reset().finish();
        },
    }

    Ok(())
}
