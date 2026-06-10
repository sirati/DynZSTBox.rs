use std::mem::size_of;
use std::ptr::{metadata, Pointee};

trait IsZeroSizedSealed {}

/// Marker trait for supported zero-sized types.
///
/// The crate implements this trait for every sized type `T` for which
/// `size_of::<T>() == 0`. The implementation uses a const-generic array bound,
/// so non-zero-sized types cannot satisfy it.
///
/// This trait is object-safe and is intended to appear as a supertrait on the
/// traits you want to erase into [`DynZSTLifetime`](crate::DynZSTLifetime).
/// The object-safe marker lets APIs state "every implementation of this trait
/// is zero-sized" without requiring the extension methods from
/// [`IsZeroSizedExt`].
pub trait IsZeroSized: IsZeroSizedSealed {}

/// Extension trait for concrete zero-sized types.
///
/// This extension trait is restricted to `Sized + Copy` zero-sized types and
/// provides helper constructors and metadata accessors used by the crate.
/// It is intentionally not object-safe; use [`IsZeroSized`] as the trait-object
/// supertrait and [`IsZeroSizedExt`] on concrete generic parameters.
pub trait IsZeroSizedExt: IsZeroSized + Sized + Copy {
    /// Return a well-aligned dangling pointer to `Self`.
    ///
    /// The pointer must only be used for zero-sized operations. It does not
    /// refer to allocated storage.
    fn zst_ptr<'a>() -> *const Self {
        let zst_ptr: *const Self = std::ptr::dangling();
        zst_ptr
    }

    /// Return a shared reference to a zero-sized `Self`.
    ///
    /// The reference is derived from a dangling pointer. This is only valid for
    /// zero-sized types, which is guaranteed by the trait bounds. The reference
    /// is useful for extracting metadata or calling methods that do not read
    /// instance storage.
    fn zst_ref<'a>() -> &'a Self {
        let zst_ptr: *const Self = std::ptr::dangling();
        unsafe { &*zst_ptr }
    }

    /// Construct a value of the zero-sized type.
    fn new_zst() -> Self {
        *Self::zst_ref()
    }

    /// Return the pointer metadata for `Self`.
    ///
    /// For sized zero-sized types this is normally `()`. The method exists so
    /// code can work uniformly with the [`Pointee`] metadata API.
    fn metadata() -> <Self as Pointee>::Metadata {
        metadata(Self::zst_ref())
    }
}

/// Blanket impl for all types that are zero-sized.
impl<T> IsZeroSizedSealed for T
where
    [u8; size_of::<T>()]: ZeroLenArray,
    T: Sized,
{
}
impl<T: IsZeroSizedSealed> IsZeroSized for T {}

impl<T: IsZeroSized + Sized + Copy> IsZeroSizedExt for T {}
//impl<T: IsZeroSized> DynIsZeroSizedSealed for T{}

/// Helper trait that only exists for `[u8; 0]`.
trait ZeroLenArray {}

impl ZeroLenArray for [u8; 0] {}
