use {
    std::{
        sync::RwLock,
        mem::MaybeUninit
    },
    lazy_static::*,
    rayon::*,
};

const DEFAULT_NUM_THREADS: usize = 1;
lazy_static! {
    static ref GLOBAL_POOL: RwLock<(usize, Option<ThreadPool>)> = {
        RwLock::new((0, None))
    };
}

pub fn num_threads(num_threads: usize) {
    if GLOBAL_POOL.read().unwrap().0 != num_threads {
        GLOBAL_POOL.write().unwrap().0 = num_threads;
        GLOBAL_POOL.write().unwrap().1 =
            Some(ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap()
            );
    }
}

pub (crate) fn global_install<OP: FnOnce() -> R + Send, R: Send>(op: OP) -> R {
    if GLOBAL_POOL.read().unwrap().0 == 0 {
        num_threads(DEFAULT_NUM_THREADS);
    }
    GLOBAL_POOL.read().unwrap().1.as_ref().unwrap().install(op)
}