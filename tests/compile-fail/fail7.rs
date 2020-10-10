use trait_union::trait_union;

trait F { }

impl F for &str { }

trait_union! {
    union U<'a>: F+'a = &'a str;
}

fn f(s: &'_ str, u: &mut U<'_>) {
    *u = U::new(s);
}

fn main() {
}
