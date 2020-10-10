use trait_union::trait_union;
use std::fmt::Debug;

trait_union! {
    union U<'a>: Debug = u8;
}

fn main() { }
