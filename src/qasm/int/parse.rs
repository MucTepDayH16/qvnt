use crate::math::R;

pub (crate) fn parse(arg: &String) -> Option<R> {
    arg[..].trim().parse::<R>().ok()
}