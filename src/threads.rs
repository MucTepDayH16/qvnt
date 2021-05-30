use {
    std::mem::MaybeUninit,
    rayon::*,
};

const DEFAULT_NUM_THREADS: usize = 1;
static mut GLOBAL_POOL: (bool, MaybeUninit<ThreadPool>) = (false, MaybeUninit::uninit());

pub fn qvnt_num_threads(num_threads: usize) {
    unsafe {
        GLOBAL_POOL.0 = true;
        GLOBAL_POOL.1 = MaybeUninit::new(ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap());
    }
}

#[inline(always)]
pub(crate) fn global_install<OP: FnOnce() -> R + Send, R: Send>(op: OP) -> R {
    unsafe {
        if !GLOBAL_POOL.0 {
            GLOBAL_POOL.0 = true;
            GLOBAL_POOL.1 = MaybeUninit::new(ThreadPoolBuilder::new()
                .num_threads(DEFAULT_NUM_THREADS)
                .build().unwrap());
        }
        (&*GLOBAL_POOL.1.as_ptr()).install(op)
    }
}