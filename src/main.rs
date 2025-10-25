
#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
#![feature(derive_coerce_pointee)]
#![feature(unsize)]
#![feature(core_intrinsics)]
#![feature(ptr_alignment_type)]
#![feature(coerce_unsized)]

mod same_type {
    trait Sealed<T1: ?Sized, T2: ?Sized> {}

    pub trait SameType<T>: Sealed<T, Self> + Sealed<Self, T> {}

    impl<T> Sealed<T, T> for T {}
    impl<T> SameType<T> for T {}


}

mod same_size {
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
}

use std::marker::{CoercePointee, Unsize};
use same_size::IsZeroSized;
use std::{fmt::Debug, marker::PhantomData, mem::{size_of, size_of_val}, ptr, ptr::{DynMetadata, from_raw_parts}};
use std::fmt::{Display, Formatter};
use std::intrinsics::vtable_size;
use std::ops::{CoerceUnsized, Deref};
use std::ptr::{metadata, NonNull, Pointee};
use crate::same_size::{IsZeroSizedExt};
use crate::same_type::SameType;

/// 1. Restrict the trait to only ZSTs.
pub trait DebugZST: IsZeroSized+Debug {
    fn foo(&self) -> String {
        "test".to_string()
    }
    fn foo2(&self) -> String {
        format!{"{self:?}"}
    }
}
impl<T: IsZeroSizedExt + Debug> DebugZST for T {}

#[derive(Debug, Copy, Clone)]
struct ZST;
#[derive(Debug, Copy, Clone)]
struct ZST2;


#[derive(Clone, Copy)]
#[repr(transparent)]
struct DynZSTLifetime<'trait_lifetime, TDyn>
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

type DynZST<TDyn> = DynZSTLifetime<'static, TDyn>;

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

fn print_stuff(dyn_zst: impl Into<DynZST<dyn DebugZST>>) {
    let dynz = dyn_zst.into();
    dbg!(dynz.foo());
    dbg!(dynz.foo2());
    dbg!(&dynz);
    dbg!(size_of_val(&dynz));
    dbg!(size_of_val(&*dynz));
}




fn main() {
    print_stuff(&ZST);
    print_stuff(&ZST2);

    //println!("size_of::<VTableOnly>() = {}", size_of::<DynZST<dyn DebugZST>>()); // ✅ should be 1 usize
    //let z = DynZST::<ZST>::new();
    println!("1");
    let dyn_ref: &dyn DebugZST = &ZST;//z.deref();
    println!("2");

    println!("dyn_ref  = {:?}, size_of_val(dyn_ref ) = {}", dyn_ref,  size_of_val(dyn_ref));
    println!("3");
    let dynz: DynZST<dyn DebugZST> =  DynZST::new(ZST); //*z;
    dbg!(dynz.foo());
    dbg!(dynz.foo2());
    println!("dynz = {:?}, size_of(dynz) = {}, size_of_val(dynz) = {}", dynz, size_of::<DynZST<dyn DebugZST>>(), size_of_val(&dynz));
    println!("4");
    let dyn_ref2 = &*dynz;
    println!("5");
    println!("dyn_ref2 = {:?}, size_of_val(dyn_ref2) = {}", dyn_ref2, size_of_val(dyn_ref2));

}
