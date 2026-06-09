# dynzst

`dynzst` provides compact trait-object wrappers for zero-sized types.

The crate stores dynamic metadata for zero-sized implementors and reconstructs
references to the requested trait object type on demand. It also includes
`DynZSTVec`, a vector-like collection of zero-sized trait objects.

## Nightly Rust

This crate currently requires nightly Rust because it uses unstable pointer and
generic constant expression features.

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

impl<T: IsZeroSizedExt + PluginImpl> Plugin for T {
    fn name(&self) -> &'static str {
        self.plugin_name()
    }
}

trait PluginImpl {
    fn plugin_name(&self) -> &'static str;
}

struct Example;

impl PluginImpl for Example {
    fn plugin_name(&self) -> &'static str {
        "example"
    }
}

let plugin: DynZSTBox<dyn Plugin> = DynZSTBox::new(Example);
assert_eq!(plugin.name(), "example");
```

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.
