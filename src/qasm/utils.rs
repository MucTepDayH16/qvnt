pub trait ToOwnedError: std::error::Error {
    type OwnedError: std::error::Error;

    fn own(self) -> Self::OwnedError;
}

pub trait DropExt {
    fn drop(self);
}