law int_reflexive(x: Int): x == x
proof int_reflexive(x: Int) {
    by test property
}

law list_reflexive(xs: List<Int>): xs == xs
proof list_reflexive(xs: List<Int>) {
    by test property
}

law option_reflexive(opt: Option<Int>): opt == opt
proof option_reflexive(opt: Option<Int>) {
    by test property
}
