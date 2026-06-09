use crate::same_size::IsZeroSizedExt;
use crate::same_type::SameType;
use std::fmt::{Debug, Display, Formatter};
use std::marker::{PhantomData, Unsize};
use std::ops::Deref;
use std::ptr;
use std::ptr::{from_raw_parts, metadata, DynMetadata, Pointee};

#[derive(Clone, Copy)]
#[repr(transparent)]
/// Metadata-only handle to a dynamic zero-sized trait object.
///
/// `DynZSTLifetime` stores only the metadata of `TDyn`, such as a trait object
/// vtable. Dereferencing reconstructs a shared reference to `TDyn` using a
/// synthetic zero-sized data pointer and the stored metadata.
///
/// The lifetime parameter represents the lifetime for which the captured trait
/// object metadata is valid. [`DynZSTBox`] is the `'static` alias used when the
/// concrete zero-sized implementor is `'static`.
///
/// # Storage model
///
/// This type does not allocate and does not own a concrete value. It owns the
/// ability to reconstruct a shared dynamic reference to a zero-sized
/// implementor. Copying or cloning the value copies only the stored metadata.
///
/// # Safety model
///
/// The public constructors require either a zero-sized concrete value
/// ([`new`](Self::new)) or an existing dynamic reference ([`with_dyn`](Self::with_dyn)).
/// Dereferencing never reads from the synthetic data pointer; it only lets the
/// trait-object vtable dispatch methods for a type whose value has no storage.
pub struct DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    dyn_meta: <TDyn as Pointee>::Metadata,
    _marker_dyn: PhantomData<&'trait_lifetime TDyn>,
}

// impl<'trait_lifetime, T, TDyn> From<&T> for DynZSTLifetime<'trait_lifetime, TDyn>
// where
//     <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
//     TDyn: ?Sized + Pointee + 'trait_lifetime,
//     T: Unsize<TDyn> + IsZeroSizedExt + 'trait_lifetime,
//     <T as Pointee>::Metadata: SameType<()>,
// {
//     fn from(value: &T) -> Self {
//         Self::new(*value)
//     }
// }

trait IntoZSTBox<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    fn into_zst_box(&self) -> DynZSTLifetime<'trait_lifetime, TDyn>;
}
impl<'trait_lifetime, T, TDyn> IntoZSTBox<'trait_lifetime, TDyn> for T
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
    T: Unsize<TDyn> + IsZeroSizedExt + 'trait_lifetime,
    <T as Pointee>::Metadata: SameType<()>,
{
    fn into_zst_box(&self) -> DynZSTLifetime<'trait_lifetime, TDyn> {
        DynZSTLifetime::with_dyn(self)
    }
}

impl<'trait_lifetime, TDyn> From<&dyn IntoZSTBox<'trait_lifetime, TDyn>>
    for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    fn from(value: &dyn IntoZSTBox<'trait_lifetime, TDyn>) -> Self {
        value.into_zst_box()
    }
}

impl<'trait_lifetime, TDyn> From<&TDyn> for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    fn from(value: &TDyn) -> Self {
        Self::with_dyn(value)
    }
}

impl<'trait_lifetime, TDyn> From<Box<TDyn>> for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    fn from(value: Box<TDyn>) -> Self {
        Self::with_dyn(&*value)
    }
}

impl<'trait_lifetime, TDyn> Debug for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (&self.deref() as &dyn Debug).fmt(f)
    }
}

impl<'trait_lifetime, TDyn> Display for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (&self.deref() as &dyn Display).fmt(f)
    }
}

/// A `'static` metadata-only handle for zero-sized trait objects.
///
/// This is the common alias for storing type-level or marker-like implementors
/// whose erased trait object does not borrow non-static data.
pub type DynZSTBox<TDyn> = DynZSTLifetime<'static, TDyn>;

impl<'trait_lifetime, TDyn> DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    /// Create a metadata-only handle from a zero-sized concrete value.
    ///
    /// The value is consumed only to infer and capture the metadata of `TDyn`.
    /// No storage is retained because `T` must be zero-sized.
    ///
    /// `T: Unsize<TDyn>` is what allows a concrete zero-sized implementor to be
    /// coerced to the requested trait object type. `T: IsZeroSizedExt` is the
    /// storage invariant that makes it valid to discard the concrete value and
    /// keep only the resulting metadata.
    pub fn new<T>(value: T) -> Self
    where
        T: Unsize<TDyn> + IsZeroSizedExt + 'trait_lifetime,
    {
        Self::with_dyn(&value)
    }

    /// Create a metadata-only handle from an existing dynamic reference.
    ///
    /// The resulting handle stores the reference metadata and uses
    /// `'trait_lifetime` to represent the validity of the captured trait object
    /// metadata.
    ///
    /// This constructor is useful when coercion to the dynamic type has already
    /// happened, for example when building from `&dyn Trait`.
    pub fn with_dyn(value: &TDyn) -> Self {
        let ptr_tdyn = value as *const TDyn;
        let dyn_meta = metadata(ptr_tdyn);

        DynZSTLifetime::<TDyn> {
            dyn_meta,
            _marker_dyn: PhantomData,
        }
    }
}

impl<'trait_lifetime, TDyn> Deref for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    type Target = TDyn;

    fn deref(&self) -> &Self::Target {
        // Safety: `dyn_meta` was captured from a valid `TDyn` reference. The
        // erased concrete type is zero-sized when built through `new`, and
        // `with_dyn` preserves the caller-provided dynamic metadata. Shared
        // references to zero-sized values do not require backing storage to be
        // read or written; the data pointer only needs to be non-null and
        // aligned well enough for the reconstructed ZST reference. `1 << 15` is
        // a power-of-two address chosen to satisfy ordinary ZST alignments.
        let zst_ptr: *const () = ptr::without_provenance(1 << 15);
        let dyn_ptr: *const TDyn = from_raw_parts(zst_ptr, self.dyn_meta);
        unsafe { &*dyn_ptr }
    }
}
