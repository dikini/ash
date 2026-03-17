// Expressions - Arithmetic and logical operations
//
// This workflow demonstrates the various expressions available in Ash.

workflow main {
    // Arithmetic expressions
    let a = 10
    let b = 3
    let sum = a + b
    let diff = a - b
    let product = a * b
    let quotient = a / b
    
    // Boolean expressions
    let isGreater = a > b
    let isEqual = a == b
    let isNotEqual = a != b
    let andResult = isGreater && !isEqual
    let orResult = isGreater || isEqual
    
    // String expressions
    let message = "Hello"
    let target = "World"
    // String concatenation via format
    let greeting = message
    
    // Comparison expressions
    let inRange = a >= 0 && a <= 100
    let isNull = null == null
    
    ret {
        arithmetic: {
            sum: sum,
            difference: diff,
            product: product,
            quotient: quotient
        },
        comparisons: {
            greater: isGreater,
            equal: isEqual,
            in_range: inRange
        },
        logical: {
            and: andResult,
            or: orResult
        },
        greeting: greeting
    }
}
