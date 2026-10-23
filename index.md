# Ash — Explore the language

> A statically typed functional language where types, effects, and resources are declared in the program. Read a complete program, take the tour, and explore the current implementation.

Page: https://dikini.github.io/ash/  
Authority: website-guidance  
Source revision: c3f17060693478cb52ff73acc0568a51fb6337fa

The Ash programming language

[In active development](/ash/status/)











An evolving functional language that pairs expressiveness with assurance. The types are the contract; the code must keep it.



[Take the language tour](/ash/learn/tour/)[Install Ash](/ash/docs/install/)



The command-line tool is currently built from source.







## A request, and its handler



effects.ash



```ash
effect Ask {
    answer : Unit -> Int
}

calculate : Unit -> Int ! {Ask}
calculate () = (perform Ask.answer ()) + 2

main : Unit -> Int
main () = handle (calculate ()) with Ask {
    answer () => resume 40
}
```





```text
42 : Int
```





`calculate` requests a value. The handler supplies `40`, and the calculation adds `2`.

[Walk through this program](/ash/learn/tour/#effects-and-handlers)









Discover



## Declared, not discovered



Data, requests, and resources shape the program.

[Discover the language](/ash/discover/)



Data types

Nominal data types define the possible cases. Generics and interfaces carry the expressiveness; local inference saves effort, and the type checker keeps it honest.





Effects and handlers

Effect rows describe what a computation can request. Lexical handlers decide how those requests are answered.





Scoped resources

Resource checking constrains where access can be used, including access captured by a function.











Learn



## Look, learn, and see



Twenty minutes, a course of study, or the final word — choose your depth.





[### Take the short tour Three complete programs covering functions, data, and effects. Assumes some programming experience. Read the tour](/ash/learn/tour/)[### Study Ash Work through connected lessons and worked cases, beginning with values and functions. Open the study guide](/ash/study/)[### Read the specification Look up syntax, type checking, and semantics in the language reference. Open the reference](/ash/spec/)









Project status



## Working implementation. Ongoing research.







Ash has an executable implementation and a language specification. Connecting programs with properties and evidence remains a research goal. Stability and production readiness are not promised.

[Read the capabilities and limitations](/ash/status/)









Diaries



## Developer diaries



Experiments, implementation questions, and work in progress between milestones.

[Read developer diaries](/ash/diaries/)





Updates



## Latest update

[18 October 2026 ### A public home for an evolving language The first website version brings discovery, language status, and study entry points together.](/ash/updates/first-version/)[All updates](/ash/updates/)
