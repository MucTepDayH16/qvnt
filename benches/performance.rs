use criterion::*;
use qvnt::{
    backend::{multi_thread::*, single_thread::*, *},
    prelude::*,
};

fn bench_op(q_num: usize) -> MultiOp {
    op::qft(0b0111 << (q_num - 4)) * op::qft(0b1110 << (q_num - 4))
}

fn perf_test(q_num: usize, builder: impl BackendBuilder) {
    let mut reg = QReg::with_builder(q_num, builder);
    let op = bench_op(q_num);

    reg.apply(&op);

    let mask = 0b100;
    assert_eq!(reg.measure_mask(mask).get() & !mask, 0);
}

fn performance(c: &mut Criterion) {
    for qu_num in [18, 19, 20] {
        c.bench_function(format!("evaluate_qu{qu_num}_single").as_str(), |b| {
            b.iter(|| perf_test(black_box(qu_num), black_box(SingleThreadBuilder)))
        });
        for th_num in 1..=rayon::current_num_threads() {
            c.bench_function(format!("evaluate_qu{qu_num}_th{th_num}").as_str(), |b| {
                b.iter(|| {
                    perf_test(
                        black_box(qu_num),
                        black_box(MultiThreadBuilder::with(th_num)),
                    )
                })
            });
        }
    }
}

criterion_group!(benches, performance);
criterion_main!(benches);
