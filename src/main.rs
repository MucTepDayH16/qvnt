use {
    std::{
        collections::BTreeMap,
        time::Instant,
    },
    rayon::prelude::*,

    qvnt::{
        operator::Op,
        register::QReg,
    },
};

fn main() {
    let mut data = BTreeMap::<usize, BTreeMap<usize, usize>>::new();

    let ops = Op::bench_circuit();

    for t_num in 8..=8 {
        let custom_pool = rayon::ThreadPoolBuilder::new().num_threads(t_num).build().unwrap();

        custom_pool.install(|| {
            println!("Running in {} threads", rayon::current_num_threads());

            for q_num in 20..=24 {
                let mut reg = QReg::new(q_num).init_state(0);

                let clock = Instant::now();

                reg.apply(&ops);
                //  println!( "{:?}", reg );
                let measured = reg.measure_mask(0b110);
                //  println!("{}", measured);

                let clock = clock.elapsed().as_millis();
                println!("\tQReg[{}] done in {}ms", q_num, clock);
                data.entry(q_num as usize).or_insert(BTreeMap::new())
                    .entry(t_num as usize).or_insert(clock as usize);
            }
        });
    }

    for ( q_num, col ) in data {
        print!( "{}", q_num );
        for ( _, time ) in col {
            print!( "\t{}", time );
        }
        println!();
    }
}
