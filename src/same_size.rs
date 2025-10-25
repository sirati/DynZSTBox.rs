use std::marker::Unsize;
use std::mem::{size_of, transmute};
use std::ptr::{metadata, DynMetadata, NonNull, Pointee};

trait IsZeroSizedSealed{}

/// A dyn-compatible trait implemented only for types whose size is 0 bytes.
pub trait IsZeroSized: IsZeroSizedSealed {}

/// dyn-incompatible extension methods for zero-sized types.
pub trait IsZeroSizedExt: IsZeroSized + Sized + Copy {

    fn zst_ptr<'a>() -> *const Self {
        let zst_ptr: *const Self = std::ptr::dangling();
        zst_ptr
    }


    fn zst_ref<'a>() -> &'a Self {
        let zst_ptr: *const Self = std::ptr::dangling();
        unsafe { &*zst_ptr }
    }

    fn new_zst() -> Self {
        *Self::zst_ref()
    }

    fn metadata() -> <Self as Pointee>::Metadata {
        metadata(Self::zst_ref())
    }
}


/// Blanket impl for all types that are zero-sized.
impl<T> IsZeroSizedSealed for T
where
    [u8; size_of::<T>()]: ZeroLenArray,
    T: Sized,
{}
impl<T: IsZeroSizedSealed> IsZeroSized for T{}

impl<T: IsZeroSized + Sized + Copy> IsZeroSizedExt for T{}
//impl<T: IsZeroSized> DynIsZeroSizedSealed for T{}

/// Helper trait that only exists for `[u8; 0]`.
trait ZeroLenArray {}

impl ZeroLenArray for [u8; 0] {}
