use trait_union::trait_union;

trait F { }

impl F for &str { }

impl<T> F for Option<T> { }

trait_union! {
    union U<'a, T: 'a>: F+'a where T: Copy = &'a str | Option<T>;
}

fn f<T: Copy>(t: T, u: &mut U<T>) {
    *u = U::new(Some(t));
}

fn main() {
}
