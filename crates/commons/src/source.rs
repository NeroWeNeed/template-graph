pub trait SourceConfiguration<D> {
    type Error;
    fn configure(self, name: impl AsRef<str>, data: &mut D) -> Result<(), Self::Error>;
}
pub trait SourceHandler<Id, Op> {
    type Error;
    fn send(&self, id: Id, op: Op) -> impl Future<Output = Result<(), Self::Error>>;
}
