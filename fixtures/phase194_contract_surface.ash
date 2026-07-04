fn requires_pos(x: Int) -> Int requires: x > 0 {
    x
}

fn ensures_ge(x: Int) -> Int ensures: result >= 0 {
    x
}

fn main() -> Int {
    let a = requires_pos(5);
    let b = ensures_ge(a);
    b
}
