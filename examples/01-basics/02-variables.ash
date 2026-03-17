// Variables - Binding values to names
//
// This workflow demonstrates variable binding using the 'let' construct.
// Patterns allow destructuring complex values.

workflow main {
    // Simple variable binding
    let name = "Ash"
    
    // Tuple destructuring
    let (x, y) = (10, 20)
    
    // Record destructuring
    let person = { name: "Alice", age: 30 }
    let { name: personName, age: personAge } = person
    
    // List pattern with rest
    let items = [1, 2, 3, 4, 5]
    let [first, second, ..rest] = items
    
    // Wildcard pattern (binds nothing)
    let _ = "ignored"
    
    ret {
        greeting: name,
        sum: x + y,
        person: personName,
        first_two: [first, second],
        remaining: rest
    }
}
