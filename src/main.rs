
#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
// #![feature(derive_coerce_pointee)]
// #![feature(unsize)]
// #![feature(core_intrinsics)]
// #![feature(ptr_alignment_type)]

mod same_type {
    trait Sealed<T1: ?Sized, T2: ?Sized> {}

    pub trait SameType<T>: Sealed<T, Self> + Sealed<Self, T> {}

    impl<T> Sealed<T, T> for T {}
    impl<T> SameType<T> for T {}


}

mod same_size {
    use crate::same_size::ptr::Thin;
use std::mem::{size_of, transmute};
    use std::ptr;
    // use std::ptr::{metadata, DynMetadata, NonNull, Pointee};

    trait IsZeroSizedSealed{}

    /// A dyn-compatible trait implemented only for types whose size is 0 bytes.
    pub trait IsZeroSized: IsZeroSizedSealed + Thin {}

    /// dyn-incompatible extension methods for zero-sized types.
    pub trait IsZeroSizedExt: IsZeroSized + Sized + Copy {

        fn zst_ptr<'a>() -> *const Self {
            let zst_ptr: *const Self = std::ptr::dangling();
            zst_ptr
        }


        fn zst_ref<'a>() -> &'a Self {
            let zst_ptr: *const Self = std::ptr::dangling();
            todo!()
            //unsafe { &*zst_ptr }
        }

        fn new_zst() -> Self {
            *Self::zst_ref()
        }

        // fn metadata() -> <Self as Pointee>::Metadata {
        //     metadata(Self::zst_ref())
        // }
    }

    /// Blanket impl for all types that are zero-sized.
    impl<T> IsZeroSizedSealed for T
    where
        [u8; size_of::<T>()]: ZeroLenArray,
        T: Sized,
    {}
    impl<T: IsZeroSizedSealed> IsZeroSized for T{}

    impl<T: IsZeroSized + Sized + Copy> IsZeroSizedExt for T{}

    /// Helper trait that only exists for `[u8; 0]`.
    trait ZeroLenArray {}

    impl ZeroLenArray for [u8; 0] {}
}

// use std::marker::{CoercePointee, Unsize};
use same_size::IsZeroSized;
use std::{fmt::Debug, marker::PhantomData, mem::{size_of, size_of_val}, ptr, /*ptr::{DynMetadata, from_raw_parts}*/};
// use std::intrinsics::vtable_size;
use std::ops::Deref;
// use std::ptr::{metadata, NonNull, Pointee};
use crate::same_size::IsZeroSizedExt;
use crate::same_type::SameType;

/// 1. Restrict the trait to only ZSTs.
pub trait DebugZST: IsZeroSized+Debug {
}
impl<T: IsZeroSizedExt + Debug> DebugZST for T {}

#[derive(Debug, Copy, Clone)]
struct ZST;
//
//
// #[derive(Clone, Copy)]
// #[repr(transparent)]
// struct DynZST<T: ?Sized+IsZeroSized>
// where
//     <T as Pointee>::Metadata: SameType<DynMetadata<T>>
// {
//     vtable: <T as Pointee>::Metadata,
// }
//
// impl<T: IsZeroSizedExt> Deref for DynZST<T>
// where
//     <T as Pointee>::Metadata: SameType<DynMetadata<T>>
// {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         T::zst_ref()
//     }
// }
//
// impl<T: IsZeroSizedExt> DynZST<T>
// where
//     <T as Pointee>::Metadata: SameType<DynMetadata<T>>
// {
//     fn new() -> Self {
//         DynZST {
//             vtable: T::metadata()
//         }
//     }
// }
//
// #[derive(Clone, Copy)]
// #[repr(transparent)]
// struct DynZSTLifetime<'trait_lifetime, T, TDyn>
// where
//         T: Unsize<TDyn>,
//         T: IsZeroSizedExt,
//         <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
//         TDyn: ?Sized + Pointee + 'trait_lifetime
//
// {
//     dyn_meta: <TDyn as Pointee>::Metadata,
//     _marker_dyn: PhantomData<&'trait_lifetime TDyn>,
//     _marker : PhantomData<&'trait_lifetime T>,
// }
//
// impl<'trait_lifetime, T, TDyn> Deref for DynZSTLifetime<'trait_lifetime, T, TDyn>
// where
//     T: Unsize<TDyn>,
//     T: IsZeroSizedExt,
//     <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
//     TDyn: ?Sized + Pointee + 'trait_lifetime
// {
//     type Target = TDyn;
//
//     fn deref(&self) -> &Self::Target {
//         /*
//             ### Safety
//             we cannot directly test the alignment of our type here as we cannot add the Sized bound
//             otherwise we are no longer dyn-compatible.
//
//             However, this type garantees that the size is zero, so we just need to create a pointer
//             that has at least the correct alignment. We assume 2^15 which is a valid pointer on
//             16bit-systems. As it's a power of two it will also be valid for all smaller alignments.
//             */
//         let zst_ptr: *const () = ptr::without_provenance(1 << 15);
//         let dyn_ptr: *const TDyn = from_raw_parts(
//             zst_ptr,
//             self.dyn_meta,
//         );
//         todo!();
//         //unsafe {&*dyn_ptr}
//     }
// }
//
// impl<'trait_lifetime, T, TDyn> DynZSTLifetime<'trait_lifetime, T, TDyn>
// where
//         for<'a> T: Unsize<TDyn>,
//         T: IsZeroSizedExt,
//         <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
//         TDyn: ?Sized + Pointee + 'trait_lifetime
// {
//     pub fn new() -> Self {
//         let t = T::new_zst();
//         let tdyn: &TDyn = &t;
//         let ptr_tdyn: *const TDyn = tdyn as *const TDyn;
//         let dyn_meta = metadata(ptr_tdyn);
//
//         DynZSTLifetime::<T, TDyn> {
//             dyn_meta: dyn_meta,
//             _marker_dyn: PhantomData,
//             _marker: PhantomData,
//         }
//     }
// }
//
// type DynZST2<T, TDyn> = DynZSTLifetime<'static, T, TDyn>;
//
//



fn main() {
    //println!("size_of::<VTableOnly>() = {}", size_of::<DynZST<dyn DebugZST>>()); // ✅ should be 1 usize
    //let z = DynZST::<ZST>::new();
    println!("1");
    let dyn_ref: &dyn DebugZST = &ZST;//z.deref();
    println!("2");

    println!("dyn_ref  = {:?}, size_of_val(dyn_ref ) = {}", dyn_ref,  size_of_val(dyn_ref));
    // println!("3");
    // let dynz: DynZST2<ZST, dyn DebugZST> =  DynZST2::new(); //*z;
    // println!("4");
    // let dyn_ref2 = &*dynz;
    // println!("5");
    // println!("dyn_ref2 = {:?}, size_of_val(dyn_ref2) = {}", dyn_ref2, size_of_val(dyn_ref2));
}
