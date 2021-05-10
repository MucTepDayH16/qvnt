use {
    criterion::*,
    qvnt::{register::*, operator::*},
};

fn qreg_multithreading(q_num: usize, t_num: usize) {
    let mut pend_ops = PendingOps::new()
        | Op::h( 0b111 )
        | Op::cx( 0b010, 0b001 )
        | Op::ch( 0b100, 0b001 )
        | Op::phi( vec![ (0b001, 1.) ], 0b001 )
        | Op::ch( 0b001, 0b100 )
        | Op::z( 0b010 );

    let custom_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(t_num).build().unwrap();

    custom_pool.install(|| {
        let mut reg = QReg::new(q_num).init_state(0);

        reg.eval(&pend_ops);
        pend_ops.clear();

        let mask = 0b100;
        assert_eq!(reg.measure(mask) & !mask, 0);
    });
}

fn qvnt_bench(crit: &mut Criterion) {
    for th_num in 1..=8 {
        for qu_num in 4..=14 {
            crit.bench_function(
                format!("evaluate_qu{}_th{}", qu_num, th_num).as_str(),
                |b| b.iter(|| {
                    qreg_multithreading(black_box(qu_num), black_box(th_num));
                })
            );
        }
    }
}

criterion_group!(benches, qvnt_bench);
criterion_main!(benches);