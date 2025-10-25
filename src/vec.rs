use std::ops::{Deref, Index};
use std::ptr::{DynMetadata, Pointee};
use std::marker::PhantomData;
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

    /// Moves all the elements of other into self, leaving other empty.
    pub fn append(&mut self, other: &mut Self) {
        self.inner.append(&mut other.inner);
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

// PartialEq / Eq by delegating to inner Vec
impl<TDyn> PartialEq for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    DynZSTBox<TDyn>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<TDyn> Eq for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    DynZSTBox<TDyn>: Eq,
{
}

// Clone by cloning the inner Vec
impl<TDyn> Clone for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    DynZSTBox<TDyn>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Extend from any item convertible Into<DynZSTBox<TDyn>>
impl<TDyn, U> Extend<U> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    U: Into<DynZSTBox<TDyn>>,
{
    fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

// From a single DynZSTBox
impl<TDyn> From<DynZSTBox<TDyn>> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
{
    fn from(b: DynZSTBox<TDyn>) -> Self {
        Self { inner: vec![b] }
    }
}

// From any Vec<U> where U can be converted into DynZSTBox<TDyn>
impl<TDyn, U> From<Vec<U>> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    U: Into<DynZSTBox<TDyn>>,
{
    fn from(v: Vec<U>) -> Self {
        let inner = v.into_iter().map(|u| u.into()).collect();
        Self { inner }
    }
}

// From a slice reference (requires Clone to duplicate elements before conversion)
impl<TDyn, U> From<&[U]> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    U: Clone + Into<DynZSTBox<TDyn>>,
{
    fn from(slice: &[U]) -> Self {
        let inner = slice.iter().cloned().map(|u| u.into()).collect();
        Self { inner }
    }
}

// From an array reference (requires Clone)
impl<TDyn, U, const N: usize> From<&[U; N]> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    U: Clone + Into<DynZSTBox<TDyn>>,
{
    fn from(arr: &[U; N]) -> Self {
        let inner = arr.iter().cloned().map(|u| u.into()).collect();
        Self { inner }
    }
}

// From Box<TDyn> if Box<TDyn> can be converted into DynZSTBox<TDyn>
impl<TDyn> From<Box<TDyn>> for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
    Box<TDyn>: Into<DynZSTBox<TDyn>>,
{
    fn from(b: Box<TDyn>) -> Self {
        Self {
            inner: vec![b.into()],
        }
    }
}

// IntoIterator for owned DynZSTVec -> yields DynZSTBox<TDyn>
impl<TDyn> IntoIterator for DynZSTVec<TDyn>
where
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
    TDyn: ?Sized + Pointee + 'static,
{
    type Item = DynZSTBox<TDyn>;
    type IntoIter = std::vec::IntoIter<DynZSTBox<TDyn>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

// Iterator over &DynZSTVec -> yields &TDyn
pub struct Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    inner: std::slice::Iter<'a, DynZSTBox<TDyn>>,
}

impl<'a, TDyn> Iterator for Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    type Item = &'a TDyn;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|b| b.deref())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl<'a, TDyn> DoubleEndedIterator for Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|b| b.deref())
    }
}
impl<'a, TDyn> ExactSizeIterator for Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// Iterator over &mut DynZSTVec -> yields a MutAccessor that remembers the index.
// The MutAccessor lets you read via Deref and replace the slot via `replace`.
pub struct MutAccessor<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    vec: *mut Vec<DynZSTBox<TDyn>>,
    index: usize,
    _marker: PhantomData<&'a mut DynZSTVec<TDyn>>,
}

impl<'a, TDyn> MutAccessor<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    /// Replace the element at this accessor's index with `item` and return the old DynZSTBox.
    pub fn replace(self, item: impl Into<DynZSTBox<TDyn>>) -> DynZSTBox<TDyn> {
        // Safety: `vec` points to the Vec owned by the IterMut; by construction no other
        // overlapping mutable access exists for this index while the accessor is alive.
        let v = unsafe { &mut *self.vec };
        std::mem::replace(&mut v[self.index], item.into())
    }

    /// Get an immutable reference to the element.
    pub fn get(&self) -> &TDyn {
        // Safety: similar reasoning as above for reading.
        let v = unsafe { & *self.vec };
        &v[self.index]
    }
}

impl<'a, TDyn> Deref for MutAccessor<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    type Target = TDyn;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

// Iteration-by-index that yields MutAccessor; uses raw pointer to avoid multiple &mut borrows.
pub struct IterMut<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    vec: *mut Vec<DynZSTBox<TDyn>>,
    next: usize,
    end: usize,
    _marker: PhantomData<&'a mut DynZSTVec<TDyn>>,
}

impl<'a, TDyn> Iterator for IterMut<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    type Item = MutAccessor<'a, TDyn>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            None
        } else {
            let idx = self.next;
            self.next += 1;
            Some(MutAccessor {
                vec: self.vec,
                index: idx,
                _marker: PhantomData,
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.end.saturating_sub(self.next);
        (len, Some(len))
    }
}

impl<'a, TDyn> DoubleEndedIterator for IterMut<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            None
        } else {
            self.end -= 1;
            Some(MutAccessor {
                vec: self.vec,
                index: self.end,
                _marker: PhantomData,
            })
        }
    }
}

impl<'a, TDyn> ExactSizeIterator for IterMut<'a, TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    fn len(&self) -> usize {
        self.end.saturating_sub(self.next)
    }
}

// IntoIterator for &mut DynZSTVec -> yields MutAccessor per element
impl<'a, TDyn> IntoIterator for &'a mut DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee + 'static,
    <TDyn as Pointee>::Metadata: SameType<DynMetadata<TDyn>>,
{
    type Item = MutAccessor<'a, TDyn>;
    type IntoIter = IterMut<'a, TDyn>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.inner.len();
        IterMut {
            vec: &mut self.inner as *mut _,
            next: 0,
            end: len,
            _marker: PhantomData,
        }
    }
}
