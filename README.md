# dynzst

`dynzst` provides compact trait-object wrappers for zero-sized types.

Use it when the concrete value behind a trait object carries no data and all
useful behavior is encoded in the type and vtable: marker states, type-level
plugins, capabilities, witnesses, and similar zero-sized objects that still
need dynamic dispatch.

A normal `Box<dyn Trait>` stores a data pointer plus metadata such as a vtable
pointer. For a zero-sized implementor, there are no instance bytes to store.
`DynZSTBox` therefore stores only the metadata. When dereferenced, it combines
that metadata with a synthetic non-null pointer and produces a shared
`&dyn Trait`.

The crate also includes `DynZSTVec`, a vector-like collection that stores one
metadata handle per element.

## Safety model

The unsafe operation in this crate is reconstructing a shared trait-object
reference from stored metadata and a synthetic data pointer. The public API
keeps this sound by enforcing these invariants:

- concrete values passed to `DynZSTBox::new` must implement `IsZeroSizedExt`,
  which means they are sized, `Copy`, and have size `0`;
- the wrapper stores only trait-object metadata for the requested dynamic type;
- dereferencing creates shared references only, never mutable references;
- zero-sized values have no instance bytes, so dereferencing does not read from
  or write to the synthetic pointer;
- the synthetic pointer is non-null and chosen with broad alignment for
  reconstructing a zero-sized reference.

Methods called through the reconstructed trait object may use the vtable and
type identity, but they must not rely on instance fields because zero-sized
values have none.

## Nightly Rust

This crate currently requires nightly Rust. In particular, it uses the
incomplete `generic_const_exprs` feature only to let the compiler prove
`size_of::<T>() == 0` at compile time for `IsZeroSized`. The metadata and
unsizing APIs used by the crate are also nightly-only today.

With the included Nix development shell:

```sh
nix develop
cargo test
```

Without Nix:

```sh
cargo +nightly test
```

## Example

```rust
#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
#![feature(unsize)]

use dynzst::{DynZSTBox, IsZeroSized, IsZeroSizedExt};

trait Plugin: IsZeroSized {
    fn name(&self) -> &'static str;
}

trait PluginImpl {
    const NAME: &'static str;
}

impl<T: IsZeroSizedExt + PluginImpl> Plugin for T {
    fn name(&self) -> &'static str {
        T::NAME
    }
}

#[derive(Clone, Copy)]
struct Example;

impl PluginImpl for Example {
    const NAME: &'static str = "example";
}

let plugin: DynZSTBox<dyn Plugin> = DynZSTBox::new(Example);
assert_eq!(plugin.name(), "example");
```

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
