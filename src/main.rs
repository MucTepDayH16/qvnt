use {
    std::{
        collections::BTreeMap,
        time::*,
    },
    rayon::prelude::*,

    qvnt::{
        operator::{Op, PendingOps},
        register::QReg,
    },
};

fn get_time() -> u128 {
    SystemTime::now().duration_since( UNIX_EPOCH ).unwrap().as_millis()
}

fn main() {
    let mut data = BTreeMap::<usize, BTreeMap<usize, usize>>::new();

    let mut pend_ops = PendingOps::new()
        | Op::h( 0b111 )
        | Op::cx( 0b010, 0b001 )
        | Op::ch( 0b100, 0b001 )
        | Op::phi( vec![ (0b010, 1.) ], 0b001 )
        | Op::ch( 0b001, 0b100 )
        | Op::z( 0b010 )
        ;

    for t_num in 8..=8 {
        let custom_pool = rayon::ThreadPoolBuilder::new().num_threads(t_num).build().unwrap();

        custom_pool.install(|| {
            println!("Running in {} threads", rayon::current_num_threads());

            for q_num in 10..=16 {
                let mut reg = QReg::new(q_num).init_state(0);

                let clock = get_time();

                reg.eval(&mut pend_ops);

                //  println!( "{:?}", reg );
                //  println!( "{:?}", reg.sample_all( 1024 ) );
                let x = reg.measure(0b100);

                let clock = get_time() - clock;
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
