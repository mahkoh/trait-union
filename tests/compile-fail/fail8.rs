use trait_union::trait_union;

trait F { }

impl F for &str { }

trait_union! {
    union U<'a>: F+'a = &'a str;
}

fn f<'a>(u: &U<'a>) {
    let _: &(dyn F + 'static) = &**u;
}

fn main() {
}
