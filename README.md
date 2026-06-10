# dynzst

`dynzst` provides a thin-pointer for dyn Trait + IsZeroSized. With other words
it allows storing ZST with dynamic dispatch in only half the bytes, namely one usize.

This is useful for marker states, type-level plugins, witnesses, capabilities,
and other ZSTs where the type does all the work but you still want dynamic
dispatch.

A normal `&dyn Trait` or `Box<dyn Trait>` is two words: data pointer and vtable.
For a ZST the data pointer is not carrying data. `DynZSTBox` stores the vtable
metadata only. On deref it builds a temporary data pointer with the alignment
from the vtable and combines both back into `&dyn Trait`.

`DynZSTVec` is the same idea for many values: a `Vec` of vtables instead of a
`Vec` of fat pointers.

## Safety model

The unsafe part is reconstructing `&dyn Trait` from only vtable metadata.
`DynZSTBox::new` proves at compile time that the concrete type is a ZST.
`DynZSTBox::with_dyn` is for already-erased values and checks the vtable size at
runtime. If the erased value is not size zero, it panics before storing the
metadata.

- `DynZSTBox::new` requires `IsZeroSizedExt`, so the concrete type is sized,
  `Copy`, and `size_of::<T>() == 0`.
- `DynZSTBox::with_dyn` requires the erased vtable size to be `0`.
- The stored value is only `DynMetadata<dyn Trait>`.
- Deref does not read any instance bytes, because there are none.
- The synthetic pointer is non-null and aligned with `DynMetadata::align_of`.

Methods may use the vtable and the concrete type. They cannot rely on instance
fields, because a ZST has no instance fields with storage.

## Nightly Rust

This crate requires nightly Rust.

The incomplete `generic_const_exprs` feature is only used to let the compiler
prove `size_of::<T>() == 0` for `IsZeroSized`. The pointer metadata and unsizing
APIs are also nightly-only today.

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
