use crate::zst_box::DynZSTBox;
use std::marker::PhantomData;
use std::ops::{Deref, Index};
use std::ptr::{DynMetadata, Pointee};

/// Vector-like storage for dynamic zero-sized trait objects.
///
/// `DynZSTVec<TDyn>` stores a `Vec<DynZSTBox<TDyn>>`. Each element is therefore
/// only the metadata needed to reconstruct a shared `&TDyn`; no concrete
/// element storage is allocated.
///
/// This is useful when the list should preserve which zero-sized implementor
/// was inserted while still presenting every element through the same dynamic
/// trait-object interface.
///
/// ```rust
/// #![feature(generic_const_exprs)]
/// #![feature(ptr_metadata)]
/// #![feature(unsize)]
///
/// use dynzst::{vec::DynZSTVec, IsZeroSized, IsZeroSizedExt};
///
/// trait Step: IsZeroSized {
///     fn label(&self) -> &'static str;
/// }
///
/// trait StepImpl {
///     const LABEL: &'static str;
/// }
///
/// impl<T: IsZeroSizedExt + StepImpl> Step for T {
///     fn label(&self) -> &'static str {
///         T::LABEL
///     }
/// }
///
/// #[derive(Clone, Copy)]
/// struct Parse;
/// impl StepImpl for Parse {
///     const LABEL: &'static str = "parse";
/// }
///
/// #[derive(Clone, Copy)]
/// struct Emit;
/// impl StepImpl for Emit {
///     const LABEL: &'static str = "emit";
/// }
///
/// let steps = DynZSTVec::from([&Parse as &dyn Step, &Emit]);
/// let labels: Vec<_> = steps.iter().map(|step| step.label()).collect();
/// assert_eq!(labels, ["parse", "emit"]);
/// ```
pub struct DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    inner: Vec<DynZSTBox<TDyn>>,
}

impl<TDyn> DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    /// Create an empty collection.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Create an empty collection with capacity for at least `cap` elements.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: Vec::with_capacity(cap),
        }
    }

    /// Append an element to the collection.
    ///
    /// The item can be any type that converts into [`DynZSTBox<TDyn>`], such
    /// as an existing metadata handle, a boxed dynamic trait object, or a
    /// dynamic reference supported by the crate's conversion impls.
    pub fn push(&mut self, item: impl Into<DynZSTBox<TDyn>>) {
        self.inner.push(item.into());
    }

    /// Move all elements from `other` into `self`, leaving `other` empty.
    pub fn append(&mut self, other: &mut Self) {
        self.inner.append(&mut other.inner);
    }

    /// Insert an element at `index` and return a reference to the inserted item.
    ///
    /// Panics if `index` is greater than the current length, matching
    /// [`Vec::insert`].
    pub fn insert(&mut self, index: usize, item: impl Into<DynZSTBox<TDyn>>) -> &TDyn {
        self.inner.insert(index, item.into());
        // safe because we just inserted
        self.get(index).unwrap()
    }

    /// Get an element by index.
    ///
    /// The returned reference is reconstructed from the stored metadata.
    pub fn get(&self, index: usize) -> Option<&TDyn> {
        self.inner.get(index).map(|b| b.deref())
    }

    /// Return the number of elements in the collection.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Return `true` if the collection contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Remove all elements from the collection.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterate over shared references to the stored dynamic zero-sized objects.
    ///
    /// Each item is reconstructed from the metadata stored at that position.
    pub fn iter(&self) -> impl Iterator<Item = &TDyn> + '_ {
        self.inner.iter().map(|b| b.deref())
    }
}

// Allow indexing like vec[idx] -> &dyn DebugZST
impl<TDyn> Index<usize> for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    type Output = TDyn;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            None => panic!(
                "Index {} out of bounds for DynZSTVec of length {}",
                index,
                self.len()
            ),
            Some(result) => result,
        }
    }
}

// PartialEq / Eq by delegating to inner Vec
impl<TDyn> PartialEq for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
    DynZSTBox<TDyn>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<TDyn> Eq for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
    DynZSTBox<TDyn>: Eq,
{
}

// Clone by cloning the inner Vec
impl<TDyn> Clone for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    fn from(b: DynZSTBox<TDyn>) -> Self {
        Self { inner: vec![b] }
    }
}

// From any Vec<U> where U can be converted into DynZSTBox<TDyn>
impl<TDyn, U> From<Vec<U>> for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
    U: Clone + Into<DynZSTBox<TDyn>>,
{
    fn from(arr: &[U; N]) -> Self {
        let inner = arr.iter().cloned().map(|u| u.into()).collect();
        Self { inner }
    }
}

impl<TDyn, const N: usize> From<[&TDyn; N]> for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    fn from(arr: [&TDyn; N]) -> Self {
        let mut result = Self::with_capacity(N);
        for item in arr {
            result.push(item);
        }

        result
    }
}

// From Box<TDyn> if Box<TDyn> can be converted into DynZSTBox<TDyn>
impl<TDyn> From<Box<TDyn>> for DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    type Item = DynZSTBox<TDyn>;
    type IntoIter = std::vec::IntoIter<DynZSTBox<TDyn>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

// Iterator over &DynZSTVec -> yields &TDyn
/// Shared iterator over a [`DynZSTVec`].
///
/// The iterator yields `&TDyn` values reconstructed from each stored metadata
/// handle.
pub struct Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    inner: std::slice::Iter<'a, DynZSTBox<TDyn>>,
}

impl<'a, TDyn> Iterator for Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|b| b.deref())
    }
}
impl<'a, TDyn> ExactSizeIterator for Iter<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// Iterator over &mut DynZSTVec -> yields a MutAccessor that remembers the index.
// The MutAccessor lets you read via Deref and replace the slot via `replace`.
/// Mutable-position accessor yielded by [`IterMut`].
///
/// Dynamic zero-sized objects do not expose mutable references to concrete
/// values. Instead, this accessor allows reading the current dynamic reference
/// and replacing the metadata handle stored at the current position.
pub struct MutAccessor<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    vec: *mut Vec<DynZSTBox<TDyn>>,
    index: usize,
    _marker: PhantomData<&'a mut DynZSTVec<TDyn>>,
}

impl<'a, TDyn> MutAccessor<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    /// Replace this element and return the previous metadata handle.
    ///
    /// This changes which zero-sized implementor is represented at this slot.
    /// It does not mutate a concrete object because no concrete object storage
    /// exists in the collection.
    pub fn replace(self, item: impl Into<DynZSTBox<TDyn>>) -> DynZSTBox<TDyn> {
        // Safety: `vec` points to the Vec owned by the IterMut; by construction no other
        // overlapping mutable access exists for this index while the accessor is alive.
        let v = unsafe { &mut *self.vec };
        std::mem::replace(&mut v[self.index], item.into())
    }

    /// Get a shared reference to the current element.
    pub fn get(&self) -> &TDyn {
        // Safety: similar reasoning as above for reading.
        let v = unsafe { &*self.vec };
        &v[self.index]
    }
}

impl<'a, TDyn> Deref for MutAccessor<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    type Target = TDyn;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

// Iteration-by-index that yields MutAccessor; uses raw pointer to avoid multiple &mut borrows.
/// Mutable iterator over a [`DynZSTVec`].
///
/// Each yielded [`MutAccessor`] can inspect or replace one stored metadata
/// handle. It deliberately does not yield `&mut TDyn`: the collection stores
/// metadata, not concrete values, and the reconstructed trait objects are
/// shared zero-sized references.
pub struct IterMut<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    vec: *mut Vec<DynZSTBox<TDyn>>,
    next: usize,
    end: usize,
    _marker: PhantomData<&'a mut DynZSTVec<TDyn>>,
}

impl<'a, TDyn> Iterator for IterMut<'a, TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
{
    fn len(&self) -> usize {
        self.end.saturating_sub(self.next)
    }
}

// IntoIterator for &mut DynZSTVec -> yields MutAccessor per element
impl<'a, TDyn> IntoIterator for &'a mut DynZSTVec<TDyn>
where
    TDyn: ?Sized + Pointee<Metadata = DynMetadata<TDyn>> + 'static,
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
