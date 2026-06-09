trait Sealed<T1: ?Sized, T2: ?Sized> {}

/// Compile-time proof that `Self` and `T` are the same type.
///
/// This trait is only implemented for `T == Self`. It is used internally to
/// constrain pointer metadata types without exposing implementation details in
/// public APIs. For example, [`DynZSTLifetime`](crate::DynZSTLifetime) requires
/// the pointee metadata of `TDyn` to be the same type as
/// `DynMetadata<TDyn>`, which rules out unexpected metadata shapes.
pub trait SameType<T>: Sealed<T, Self> + Sealed<Self, T> {}

impl<T> Sealed<T, T> for T {}
impl<T> SameType<T> for T {}
