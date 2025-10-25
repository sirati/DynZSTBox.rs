use std::fmt::{Debug, Display, Formatter};
use std::marker::{PhantomData, Unsize};
use std::ops::Deref;
use std::ptr;
use std::ptr::{from_raw_parts, metadata, DynMetadata, Pointee};
use crate::same_size::IsZeroSizedExt;
use crate::same_type::SameType;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,

{
    dyn_meta: <TDyn as Pointee>::Metadata,
    _marker_dyn: PhantomData<&'trait_lifetime TDyn>,
}

//conflicting impl in core?
impl<'trait_lifetime, T, TDyn> From<&T> for DynZSTLifetime<'trait_lifetime, TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'trait_lifetime,
    T: Unsize<TDyn> + IsZeroSizedExt + 'trait_lifetime,
    <T as Pointee>::Metadata: SameType<()>,
{
    fn from(value: &T) -> Self {
        Self::new(*value)
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

pub type DynZSTBox<TDyn> = DynZSTLifetime<'static, TDyn>;

impl<'trait_lifetime, TDyn> DynZSTLifetime<'trait_lifetime, TDyn>
where
        <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
        TDyn: ?Sized + Pointee + 'trait_lifetime,
{
    pub fn new<T>(value: T) -> Self
    where
        T: Unsize<TDyn> + IsZeroSizedExt + 'trait_lifetime,
    {
        let tdyn: &TDyn = &value;
        let ptr_tdyn = tdyn as *const TDyn;
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
        /*
            ### Safety
            we cannot directly test the alignment of our type here as we cannot add the Sized bound
            otherwise we are no longer dyn-compatible.

            However, this type garantees that the size is zero, so we just need to create a pointer
            that has at least the correct alignment. We assume 2^15 which is a valid pointer on
            16bit-systems. As it's a power of two it will also be valid for all smaller alignments.
            */
        let zst_ptr: *const () = ptr::without_provenance(1 << 15);
        let dyn_ptr: *const TDyn = from_raw_parts(
            zst_ptr,
            self.dyn_meta,
        );
        unsafe {&*dyn_ptr}
    }
}