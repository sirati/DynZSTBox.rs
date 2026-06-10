//! Thin pointers for `dyn Trait + IsZeroSized`.
//!
//! `dynzst` stores ZSTs with dynamic dispatch in one word. In other words,
//! `DynZSTBox<dyn Trait>` keeps only the vtable metadata for a
//! `dyn Trait + IsZeroSized` value.
//!
//! A normal `&dyn Trait` is two words: data pointer and vtable. For a ZST the
//! data pointer is not carrying data. [`DynZSTBox`] stores the vtable metadata
//! only. On deref it builds a temporary data pointer with the alignment from
//! the vtable and combines both back into `&dyn Trait`.
//!
//! Use it for marker states, type-level plugins, witnesses, capabilities, and
//! other ZSTs where the type does all the work but you still want dynamic
//! dispatch.
//!
//! # Defining an object-safe zero-sized trait
//!
//! The trait you want to erase should inherit [`IsZeroSized`]. Implementations
//! are usually given through a blanket impl over [`IsZeroSizedExt`], which is
//! implemented by the crate only for sized zero-sized `Copy` types.
//!
//! ```rust
//! #![feature(generic_const_exprs)]
//! #![feature(ptr_metadata)]
//! #![feature(unsize)]
//!
//! use dynzst::{DynZSTBox, IsZeroSized, IsZeroSizedExt};
//!
//! trait Mode: IsZeroSized {
//!     fn name(&self) -> &'static str;
//! }
//!
//! trait ModeImpl {
//!     const NAME: &'static str;
//! }
//!
//! impl<T> Mode for T
//! where
//!     T: IsZeroSizedExt + ModeImpl,
//! {
//!     fn name(&self) -> &'static str {
//!         T::NAME
//!     }
//! }
//!
//! #[derive(Clone, Copy)]
//! struct Fast;
//!
//! impl ModeImpl for Fast {
//!     const NAME: &'static str = "fast";
//! }
//!
//! let mode: DynZSTBox<dyn Mode> = DynZSTBox::new(Fast);
//! assert_eq!(mode.name(), "fast");
//! ```
//!
//! # Why dereferencing is sound
//!
//! The unsafe part is reconstructing `&dyn Trait` from only vtable metadata.
//! [`DynZSTLifetime::new`] proves at compile time that the concrete type is a
//! ZST. [`DynZSTLifetime::with_dyn`] is for already-erased values and checks
//! the vtable size at runtime. If the erased value is not size zero, it panics
//! before storing the metadata.
//!
//! - [`DynZSTLifetime::new`] requires [`IsZeroSizedExt`], so the concrete type
//!   is sized, `Copy`, and `size_of::<T>() == 0`.
//! - [`DynZSTLifetime::with_dyn`] requires the erased vtable size to be `0`.
//! - The stored value is only [`DynMetadata`](std::ptr::DynMetadata).
//! - Deref does not read any instance bytes, because there are none.
//! - The synthetic pointer is non-null and aligned with
//!   [`DynMetadata::align_of`](std::ptr::DynMetadata::align_of).
//!
//! Methods may use the vtable and the concrete type. They cannot rely on
//! instance fields, because a ZST has no instance fields with storage.
//!
//! This crate requires nightly Rust. The incomplete `generic_const_exprs`
//! feature is only used to let the compiler prove `size_of::<T>() == 0` for
//! [`IsZeroSized`]. The pointer metadata and unsizing APIs are also
//! nightly-only today.

#![warn(missing_docs)]
#![allow(incomplete_features)]
#![allow(private_bounds)]
#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
#![feature(unsize)]

/// Traits for accepting only zero-sized types.
pub mod same_size;
/// Compile-time helper proving that two types are the same type.
pub mod same_type;
/// Vector-like storage for dynamic zero-sized trait objects.
pub mod vec;
/// Metadata-backed trait-object wrappers for zero-sized types.
pub mod zst_box;

pub use crate::same_size::{IsZeroSized, IsZeroSizedExt};
pub use crate::same_type::SameType;
pub use crate::zst_box::{DynZSTBox, DynZSTLifetime};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec::DynZSTVec;
    use std::fmt::Debug;

    /// 1. Restrict the trait to only ZSTs.
    pub trait DebugZST: IsZeroSized + Debug {
        fn foo(&self) -> String {
            "test".to_string()
        }
        fn foo2(&self) -> String {
            format!("{self:?}")
        }
    }
    impl<T: IsZeroSizedExt + Debug> DebugZST for T {}

    pub trait AlignCheck: IsZeroSized {
        fn addr_mod_align(&self) -> usize;
        fn expected_align(&self) -> usize;
    }
    impl<T: IsZeroSizedExt> AlignCheck for T {
        fn addr_mod_align(&self) -> usize {
            (self as *const T as usize) % std::mem::align_of::<T>()
        }

        fn expected_align(&self) -> usize {
            std::mem::align_of::<T>()
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct ZST;
    #[derive(Debug, Copy, Clone)]
    struct ZST2;
    #[repr(align(65536))]
    #[derive(Debug, Copy, Clone)]
    struct OverAlignedZST;

    #[test]
    fn test_debug_zst() {
        let zst = ZST;
        let dynz: DynZSTBox<dyn DebugZST> = DynZSTBox::new(zst);
        assert_eq!(dynz.foo(), "test".to_string());
        assert_eq!(dynz.foo2(), "ZST".to_string());
    }

    #[test]
    fn test_vec() {
        let vec = DynZSTVec::from([&ZST as &dyn DebugZST, &ZST2, &ZST]);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].foo(), "test".to_string());
        assert_eq!(vec[1].foo2(), "ZST2".to_string());
        assert_eq!(vec[2].foo2(), "ZST".to_string());
    }

    #[test]
    fn test_overaligned_zst() {
        let dynz: DynZSTBox<dyn DebugZST> = DynZSTBox::new(OverAlignedZST);
        assert_eq!(dynz.foo2(), "OverAlignedZST".to_string());

        let aligned: DynZSTBox<dyn AlignCheck> = DynZSTBox::new(OverAlignedZST);
        assert_eq!(aligned.expected_align(), 65536);
        assert_eq!(aligned.addr_mod_align(), 0);
    }

    #[test]
    #[should_panic(expected = "requires a zero-sized dynamic value")]
    fn with_dyn_rejects_non_zst_dynamic_value() {
        let value = String::from("not zst");
        let dyn_value: &dyn Debug = &value;
        let _ = DynZSTLifetime::<dyn Debug>::with_dyn(dyn_value);
    }
}
