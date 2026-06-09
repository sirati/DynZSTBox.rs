//! Trait-object containers for zero-sized types.
//!
//! `dynzst` is for APIs where the concrete value of a trait object carries no
//! data and all useful behavior is encoded in its type and vtable. Typical
//! examples are marker states, type-level plugins, capabilities, and other
//! zero-sized witnesses that still need dynamic dispatch.
//!
//! A normal `Box<dyn Trait>` stores two things: a data pointer and metadata
//! such as a vtable pointer. For a zero-sized implementor, the data pointer
//! does not point at meaningful storage because the value occupies no bytes.
//! [`DynZSTBox`] therefore stores only the metadata part. When dereferenced, it
//! combines that metadata with a synthetic, non-null, well-aligned pointer and
//! produces a shared `&dyn Trait`.
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
//! The unsafe operation in this crate is reconstructing a shared reference from
//! stored metadata and a synthetic data pointer. The public constructors keep
//! that operation sound by enforcing these invariants:
//!
//! - the concrete type used to create a [`DynZSTLifetime`] must implement
//!   [`IsZeroSizedExt`], so it is sized, `Copy`, and has size `0`;
//! - the wrapper stores only trait-object metadata for `TDyn`, enforced through
//!   the [`SameType`] metadata bounds;
//! - dereferencing only creates shared references, never mutable references, so
//!   there is no mutable access to aliased synthetic storage;
//! - because the concrete value is zero-sized, no bytes are read from or written
//!   to the synthetic pointer;
//! - the synthetic pointer is non-null and chosen with broad alignment for the
//!   zero-sized reference reconstruction.
//!
//! As a result, method calls dispatched through the reconstructed trait object
//! can use the vtable and type identity, but must not depend on reading
//! instance fields: zero-sized values have none.
//!
//! This crate requires nightly Rust. In particular, it uses the incomplete
//! `generic_const_exprs` feature only to let the compiler prove
//! `size_of::<T>() == 0` at compile time for [`IsZeroSized`]. The metadata and
//! unsizing APIs used by the crate are also nightly-only today.

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

    #[derive(Debug, Copy, Clone)]
    struct ZST;
    #[derive(Debug, Copy, Clone)]
    struct ZST2;

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
}
