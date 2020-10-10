use trait_union::trait_union;

trait F { }

impl F for &str { }

trait_union! {
    union U<'a>: F+'a = &'a str;
}

fn f<'a, 'b>(u: &'b U<'a>) -> &'b (dyn F + 'a) {
    &**u
}

fn main() {
}
