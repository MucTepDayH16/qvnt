use std::sync::RwLock;

use lazy_static::*;
use rayon::*;

lazy_static! {
    static ref GLOBAL_POOL: RwLock<Option<(usize, ThreadPool),>,> = RwLock::new(None);
}

fn get_current_num_threads() -> Option<usize,> {
    GLOBAL_POOL
        .read()
        .unwrap()
        .as_ref()
        .map(|(th, _,)| th,)
        .cloned()
}

fn global_install_unchecked<OP, R,>(op: OP,) -> R
where
    OP: FnOnce() -> R + Send,
    R: Send,
{
    GLOBAL_POOL
        .read()
        .unwrap()
        .as_ref()
        .map(|(_, tp,)| tp.install(op,),)
        .unwrap()
}

fn set_num_threads(num_threads: usize,) {
    *GLOBAL_POOL.write().unwrap() = Some((
        num_threads,
        ThreadPoolBuilder::new()
            .num_threads(num_threads,)
            .build()
            .unwrap(),
    ),);
}

pub(crate) fn global_install<OP, R,>(num_threads: usize, op: OP,) -> R
where
    OP: FnOnce() -> R + Send,
    R: Send,
{
    match get_current_num_threads() {
        Some(th,) if th == num_threads => {}
        _ => set_num_threads(num_threads,),
    };
    global_install_unchecked(op,)
}
