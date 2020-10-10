#![feature(untagged_unions)]

use trait_union::trait_union;
use std::cell::Cell;

trait F { }

impl F for Cell<&'_ mut u8> { }

trait_union! {
    union U<'a>: F+'a = Cell<&'a mut u8>;
}

fn f<'a, 'b: 'a>(u: &U<'b>) {
    let _: &U<'a> = u;
}

fn main() {
}
