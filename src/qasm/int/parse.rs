use meval::*;

use crate::math::*;

thread_local! {
    static EXAUSTIVE_CONTEXT: Context<'static> = {
        let mut ctx = Context::empty();
        ctx.var("pi", PI);

        ctx.func("sqrt", f64::sqrt);
        ctx.func("exp", f64::exp);
        ctx.func("ln", f64::ln);
        ctx.func("abs", f64::abs);

        ctx.func("floor", f64::floor);
        ctx.func("ceil", f64::ceil);
        ctx.func("round", f64::round);

        ctx.func2("atan2", f64::atan2);
        ctx.funcn("max", max_array, 1..);
        ctx.funcn("min", min_array, 1..);

        ctx
    }
}

pub use meval::Error;
pub type Result<T,> = std::result::Result<T, meval::Error,>;

pub(crate) fn eval_extended<'t, V: IntoIterator<Item = (&'t str, f64,),>,>(
    expr: &'t str,
    vars: V,
) -> Result<R,> {
    let mut ctx = EXAUSTIVE_CONTEXT.with(|ctx| ctx.clone(),);
    for (var, value,) in vars {
        ctx.var(var, value,);
    }

    expr.parse::<Expr>()?.eval_with_context(ctx,)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_expr() {
        let expr = "2 * pi / 16";

        assert_eq!(eval_extended(expr, None), Ok(2. * PI / 16.));
    }
}
