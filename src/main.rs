
#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
#![feature(unsize)]

mod vec;
mod same_type;
mod same_size;
mod zst_box;

use same_size::IsZeroSized;
use std::{fmt::Debug, mem::{size_of, size_of_val}};
pub use crate::zst_box::{DynZSTBox, DynZSTLifetime};
use crate::same_size::IsZeroSizedExt;
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


fn print_stuff(dyn_zst: impl Into<DynZSTBox<dyn DebugZST>>) {
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
    let dynz: DynZSTBox<dyn DebugZST> =  DynZSTBox::new(ZST); //*z;
    dbg!(dynz.foo());
    dbg!(dynz.foo2());
    println!("dynz = {:?}, size_of(dynz) = {}, size_of_val(dynz) = {}", dynz, size_of::<DynZSTBox<dyn DebugZST>>(), size_of_val(&dynz));
    println!("4");
    let dyn_ref2 = &*dynz;
    println!("5");
    println!("dyn_ref2 = {:?}, size_of_val(dyn_ref2) = {}", dyn_ref2, size_of_val(dyn_ref2));

}
