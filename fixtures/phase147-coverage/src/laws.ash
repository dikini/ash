law covered_identity(x: Int): x == x
proof covered_identity(x: Int) {
    by test "covered_law_test"
}

law uncovered_identity(x: Int): x == x
