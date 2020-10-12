#![feature(untagged_unions)]

use trait_union::trait_union_copy;

trait_union_copy! {
    union U: std::fmt::Display = u8 | String;
}

fn main() {
}
