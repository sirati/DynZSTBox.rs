use std::ops::{Deref, Index};
use std::ptr::{DynMetadata, Pointee};
use crate::same_type::SameType;
use crate::zst_box::DynZSTBox;

pub struct DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
{
    inner: Vec<DynZSTBox<TDyn>>,
}

impl<TDyn> DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
{
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Create with capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    /// Push an element (accepts anything convertible Into<DynZSTBox<dyn DebugZST>>)
    pub fn push(&mut self, item: impl Into<DynZSTBox<TDyn>>) {
        self.inner.push(item.into());
    }

    /// Insert at index and return a reference to the inserted element.
    pub fn insert(&mut self, index: usize, item: impl Into<DynZSTBox<TDyn>>) -> &TDyn {
        self.inner.insert(index, item.into());
        // safe because we just inserted
        self.get(index).unwrap()
    }

    /// Get an element by index as `&dyn DebugZST`.
    pub fn get(&self, index: usize) -> Option<&TDyn> {
        self.inner.get(index).map(|b| b.deref())
    }

    /// Number of elements.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all elements.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterator over `&dyn DebugZST`.
    pub fn iter(&self) -> impl Iterator<Item = &TDyn> + '_ {
        self.inner.iter().map(|b| b.deref())
    }

}

// Allow indexing like vec[idx] -> &dyn DebugZST
impl<TDyn> Index<usize> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
{
    type Output = TDyn;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            None => panic!("Index {} out of bounds for DynZSTVec of length {}", index, self.len()),
            Some(result) => result
        }
    }
}


