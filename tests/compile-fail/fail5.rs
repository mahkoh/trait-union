use trait_union::trait_union;

trait F { }

trait_union! {
    union U<T>: F where T: Copy+'static = T;
}

fn main() { }
