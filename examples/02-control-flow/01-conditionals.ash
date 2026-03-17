// Conditionals - Branching logic
//
// This workflow demonstrates conditional execution with if/then/else.

workflow main {
    let score = 85
    
    // Simple if/then/else
    let grade = if score >= 90 {
        "A"
    } else if score >= 80 {
        "B"
    } else if score >= 70 {
        "C"
    } else if score >= 60 {
        "D"
    } else {
        "F"
    }
    
    // Using conditionals in workflow control
    if grade == "A" || grade == "B" {
        ret {
            score: score,
            grade: grade,
            message: "Great job!",
            passed: true
        }
    } else if grade == "C" {
        ret {
            score: score,
            grade: grade,
            message: "You passed.",
            passed: true
        }
    } else {
        ret {
            score: score,
            grade: grade,
            message: "Needs improvement.",
            passed: false
        }
    }
}
