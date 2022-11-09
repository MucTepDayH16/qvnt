use criterion::*;
use qvnt::{backend::single_thread::SingleThreadBuilder, prelude::*};

fn perf_test_single(q_num: usize) {
    let mut reg = QReg::with_builder(q_num, SingleThreadBuilder);

    reg.apply(&(op::qft(0b0111) * op::qft(0b1110)));

    let mask = 0b100;
    assert_eq!(reg.measure_mask(mask).get() & !mask, 0);
}

fn perf_test_multi(q_num: usize, _t_num: usize) {
    let mut reg = QReg::with_builder(q_num, SingleThreadBuilder);

    reg.apply(&(op::qft(0b0111) * op::qft(0b1110)));

    let mask = 0b100;
    assert_eq!(reg.measure_mask(mask).get() & !mask, 0);
}

fn performance(c: &mut Criterion) {
    for qu_num in [18, 19, 20] {
        c.bench_function(format!("evaluate_qu{qu_num}_single").as_str(), |b| {
            b.iter(|| perf_test_single(black_box(qu_num)))
        });
        for th_num in 1..=rayon::current_num_threads() {
            c.bench_function(format!("evaluate_qu{qu_num}_th{th_num}").as_str(), |b| {
                b.iter(|| perf_test_multi(black_box(qu_num), black_box(th_num)))
            });
        }
    }
}

criterion_group!(benches, performance);
criterion_main!(benches);
