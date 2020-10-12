// #![cfg_attr(not(test), no_std)]
#![cfg_attr(test, feature(untagged_unions))]

//! This crate provides a macro that generates a trait-union type. That is, a trait
//! object type which can contain any one of a pre-determined set of implementors.
//!
//! The generated type does not allocate. The size of the type is the size of the largest
//! variant plus some constant overhead.
//!
//! **NOTE**: As of rustc 1.47, you must enable the `untagged_unions` feature to store
//! non-[Copy] types in a trait-union. This will change
//! [soon](https://github.com/rust-lang/rust/pull/77547).
//!
//! # Example
//!
//! ```rust
//! # use trait_union::trait_union;
//! # use std::fmt::Display;
//! #
//! trait_union! {
//!     /// Container can contain either an i32, a &'static str, or a bool.
//!     union Container: Display = i32 | &'static str | bool;
//! }
//!
//! let mut container = Container::new(32);
//! assert_eq!(container.to_string(), "32");
//!
//! container = Container::new("Hello World");
//! assert_eq!(container.to_string(), "Hello World");
//!
//! container = Container::new(true);
//! assert_eq!(container.to_string(), "true");
//! ```
//!
//! The generated type has the following interface:
//!
//! ```rust,ignore
//! struct Container {
//!     /* ... */
//! }
//!
//! impl Container {
//!     fn new(value: impl ContainerVariant) -> Self { /* ... */ }
//! }
//!
//! impl Deref for Container {
//!     type Target = dyn Display + 'static;
//!     /* ... */
//! }
//!
//! impl DerefMut for Container {
//!     /* ... */
//! }
//!
//! unsafe trait ContainerVariant: Display + 'static { }
//!
//! unsafe impl ContainerVariant for i32 { }
//! unsafe impl ContainerVariant for &'static str { }
//! unsafe impl ContainerVariant for bool { }
//! ```

/// Macro that generates a trait-union type
///
/// # Syntax
///
/// Each invocation of the macro can generate an arbitrary number of trait-union types.
///
/// The syntax of each declaration is as follows:
///
/// ```txt
/// ATTRIBUTE* VISIBILITY? 'union' NAME GENERICS? ':' TRAIT_BOUNDS ('where' WHERE_CLAUSE)? '=' TYPE ('|' TYPE)* '|'? ';'
/// ```
///
/// `?` denotes an optional segment. `*` denotes 0 or more repetitions.
///
/// For example:
///
/// ```rust,ignore
/// /// MyUnion trait-union
/// pub(crate) union MyUnion<'a, T: 'a>: Debug+'a where T: Debug+Copy = &'a str | Option<T>;
/// ```
///
/// # Trait bounds
///
/// The `TRAIT_BOUNDS` segment denotes the trait that the trait-union will deref to. As
/// such, it must contain at least one trait, at most one non-auto trait, and 0 or more
/// lifetimes.
///
/// For example:
///
/// ```rust,ignore
/// Debug+Copy+'a // OK
/// 'a            // Error: No trait
/// Debug+Display // Error: More than one non-auto trait
/// ```
///
/// If you do not provide a lifetime, the `'static` lifetime will be added automatically.
/// That is, `Debug` is the same as `Debug+'static`. For example
///
/// ```rust,ignore
/// union MyUnion<'a>: Debug = &'a str;
/// ```
///
/// will not compile because `&'a str` is not `'static`. Write
///
/// ```rust,ignore
/// union MyUnion<'a>: Debug+'a = &'a str;
/// ```
///
/// instead.
///
/// # Output
///
/// The macro generates a struct with the specified name and an unsafe trait of the same
/// name plus the suffix `Variant`. For example
///
/// ```rust,ignore
/// pub(crate) union MyUnion<'a, T: 'a>: Debug+'a where T: Debug+Copy = &'a str | Option<T>
/// ```
///
/// generates
///
/// ```rust,ignore
/// pub(crate) struct MyUnion<'a, T: 'a> where T: Debug+Copy {
///     /* ... */
/// }
///
/// pub(crate) unsafe trait MyUnionVariant<'a, T: 'a>: Debug+'a where T: Debug+Copy { }
/// ```
///
/// The trait will automatically be implemented for all specified variants. The struct has
/// a single associated method:
///
/// ```rust,ignore
/// pub(crate) fn new(value: impl MyUnionVariant<'a, T>) -> Self { /* ... */ }
/// ```
///
/// The struct implements `Deref` and `DerefMut` with `Target = Debug+'a`.
pub use trait_union_proc::trait_union;

#[cfg(test)]
mod test {
    use super::trait_union;
    use std::{
        fmt,
        fmt::{Display, Formatter},
        mem,
        sync::atomic::{AtomicUsize, Ordering::Relaxed},
    };

    trait F: Display {
        fn len(&self) -> usize;

        fn set_len(&mut self, len: usize);
    }

    impl F for u8 {
        fn len(&self) -> usize {
            *self as usize
        }

        fn set_len(&mut self, len: usize) {
            *self = len as u8;
        }
    }

    impl F for String {
        fn len(&self) -> usize {
            self.len()
        }

        fn set_len(&mut self, len: usize) {
            self.truncate(len);
        }
    }

    #[repr(align(4))]
    struct X;
    impl F for X {
        fn len(&self) -> usize {
            !0
        }

        fn set_len(&mut self, len: usize) {
            X_DROP_COUNT.store(len, Relaxed);
        }
    }
    static X_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
    impl Drop for X {
        fn drop(&mut self) {
            X_DROP_COUNT.fetch_add(1, Relaxed);
        }
    }
    impl Display for X {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "X")
        }
    }

    trait_union! {
        union U: F = u8 | String | X;
    }

    #[test]
    fn test1() {
        let mut c = U::new(33);
        assert_eq!(mem::align_of_val(&*c), 1);
        assert_eq!(mem::size_of_val(&*c), 1);
        assert!(mem::align_of_val(&c) >= 4);
        assert!(mem::size_of_val(&c) >= 4);
        assert_eq!(c.len(), 33);
        c.set_len(22);
        assert_eq!(c.len(), 22);
        c = U::new("Hello World".to_string());
        assert_eq!(c.len(), 11);
        c.set_len(5);
        assert_eq!(c.len(), 5);
        assert_eq!(c.to_string(), "Hello");
        c = U::new(X);
        assert_eq!(mem::align_of_val(&*c), 4);
        assert_eq!(mem::size_of_val(&*c), 0);
        assert_eq!(c.len(), !0);
        assert_eq!(X_DROP_COUNT.load(Relaxed), 0);
        c.set_len(2);
        assert_eq!(X_DROP_COUNT.load(Relaxed), 2);
        drop(c);
        assert_eq!(X_DROP_COUNT.load(Relaxed), 3);
    }

    #[test]
    fn size() {
        assert_eq!(mem::size_of::<U>(), mem::size_of::<Option<U>>());
    }

    #[test]
    fn compile() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/compile-fail/*.rs");
        t.pass("tests/pass/*.rs");
    }

    #[test]
    fn assert_sync() {
        let _: &dyn Sync = &U::new(1);
    }
}
