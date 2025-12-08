#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
#![feature(unsize)]

pub mod vec;
pub mod same_type;
pub mod same_size;
pub mod zst_box;

pub use crate::zst_box::{DynZSTBox, DynZSTLifetime};
pub use crate::same_size::{IsZeroSized, IsZeroSizedExt};
pub use crate::same_type::SameType;

use std::{fmt::Debug};



#[cfg(test)]
mod tests {
    use crate::vec::DynZSTVec;
    use super::*;


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


