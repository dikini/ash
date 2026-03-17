// ForEach - Looping over collections
//
// This workflow demonstrates iteration with the for construct.

workflow main {
    let numbers = [1, 2, 3, 4, 5]
    let words = ["hello", "world", "ash"]
    
    // Iterate over a list and transform
    let doubled = []
    for n in numbers {
        let doubled_n = n * 2
        // In real Ash, this would accumulate
    }
    
    // Iterate with pattern matching
    let people = [
        { name: "Alice", age: 30 },
        { name: "Bob", age: 25 },
        { name: "Carol", age: 35 }
    ]
    
    // Calculate average age
    let total = 0
    let count = 0
    for { name: n, age: a } in people {
        let total = total + a
        let count = count + 1
    }
    
    // Filter with condition
    let adults = []
    for person in people {
        if person.age >= 18 {
            // Collect adult
        }
    }
    
    ret {
        numbers: numbers,
        total_age: total,
        count: count,
        average: if count > 0 { total / count } else { 0 }
    }
}
