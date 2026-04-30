// ForEach - current Ash for/do syntax.
//
// Ash currently supports `for <pattern> in <expr> do <workflow>`. This small
// example demonstrates that parser-supported loop shape without claiming list
// accumulation or collection transforms.

workflow main {
    let numbers = [1, 2, 3, 4, 5]

    for n in numbers do ret n
}
