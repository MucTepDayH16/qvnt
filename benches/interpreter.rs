use criterion::*;
use qvnt::{prelude::*, qasm::Sym};
use std::path::PathBuf;

fn run_interpreter(path: &PathBuf) {
    let source = std::fs::read_to_string(path).unwrap();
    let ast = Ast::from_source(&source[..]).unwrap();
    let int = Int::new(ast).unwrap();
    let mut sym = Sym::new(int);

    sym.reset();
    sym.finish();
}

fn interpreter(c: &mut Criterion) {
    for source in glob::glob("./qasm-rust/tests/source/*.qasm").unwrap() {
        let source = source.unwrap();
        let file_name = source.file_name().and_then(|f| f.to_str()).unwrap();
        if file_name == "qelib1.qasm" {
            continue;
        }
        c.bench_function(&format!("cirquit_{file_name}"), |b| {
            b.iter(|| run_interpreter(black_box(&source)))
        });
    }
}

criterion_group!(benches, interpreter);
criterion_main!(benches);
