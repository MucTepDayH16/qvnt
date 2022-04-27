pub trait DropExt {
    fn drop(self);
}

impl<'t> DropExt for qvnt::qasm::Int<'t> {
    fn drop(self) {
        self.into_iter_ast().for_each(DropExt::drop);
    }
}

impl<'t> DropExt for qvnt::qasm::Ast<'t> {
    fn drop(self) {
        unsafe {
            std::mem::drop(Box::from_raw(self.source() as *const str as *mut str));
        }
    }
}
