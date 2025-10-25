trait Sealed<T1: ?Sized, T2: ?Sized> {}

pub trait SameType<T>: Sealed<T, Self> + Sealed<Self, T> {}

impl<T> Sealed<T, T> for T {}
impl<T> SameType<T> for T {}


