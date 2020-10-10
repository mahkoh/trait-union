use trait_union::trait_union;

trait F { }

impl F for &str { }

trait_union! {
    union U: F = &'static str;
}

fn f(s: &str) {
    let u = U::new(s);
}

fn main() {
}
