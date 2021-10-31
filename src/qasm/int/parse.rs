use crate::math::R;

pub (crate) fn eval(arg: &String) -> Option<R> {
    arg[..].trim().parse::<R>().ok()
}