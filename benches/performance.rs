use criterion::*;
use qvnt::prelude::*;

fn perf_test(q_num: usize, t_num: usize) {
    #[cfg(feature = "cpu")]
    qvnt::num_threads(t_num);

    let mut reg = QReg::new(q_num).init_state(0);
    reg.apply(&(op::qft(0b0111) * op::qft(0b1110)));

    let mask = 0b100;
    assert_eq!(reg.measure_mask(mask).get() & !mask, 0);
}

fn performance(c: &mut Criterion) {
    #[cfg(feature = "cpu")]
    let count = rayon::current_num_threads();
    #[cfg(not(feature = "cpu"))]
    let count = 1;

    for th_num in 1..=count {
        for qu_num in [18, 19, 20] {
            c.bench_function(
                format!("evaluate_qu{}_th{}", qu_num, th_num).as_str(),
                |b| b.iter(||
                    perf_test(black_box(qu_num), black_box(th_num))
                ));
        }
    }
}

criterion_group!(benches, performance);
criterion_main!(benches);