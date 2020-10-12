# trait-union

[![crates.io](https://img.shields.io/crates/v/trait-union.svg)](http://crates.io/crates/trait-union)
[![docs.rs](https://docs.rs/trait-union/badge.svg)](http://docs.rs/trait-union)

This crate provides a macro that generates a trait-union type. That is, a trait
object type which can contain any one of a pre-determined set of implementors.

The generated type does not allocate. The size of the type is the size of the largest
variant plus some constant overhead.

**NOTE**: As of rustc 1.47, you must enable the `untagged_unions` feature to store
non-Copy types in a trait-union. This will change
[soon](https://github.com/rust-lang/rust/pull/77547).

# Example

```rust
use trait_union::trait_union;
use std::fmt::Display;

trait_union! {
    /// Container can contain either an i32, a &'static str, or a bool.
    union Container: Display = i32 | &'static str | bool;
}

let mut container = Container::new(32);
assert_eq!(container.to_string(), "32");

container = Container::new("Hello World");
assert_eq!(container.to_string(), "Hello World");

container = Container::new(true);
assert_eq!(container.to_string(), "true");
```

# Implementation

The generated type looks roughly as follows:

```rust
struct Container {
    data: union {
        variant1: i32,
        variant2: &'static str,
        variant3: bool,
    },
    vtable: *mut (),
}
```

Its size is therefore similar to the size of an `enum` with one variant per implementor.
Depending on the number of implementors, compile times should be significantly lower than
with an `enum`. The run-time performance is similar to that of `Box<dyn Trait>`. 

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.
