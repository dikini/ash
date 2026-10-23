# A short tour of Ash

> Three complete programs. A first look at values, data, and how computations ask for things.

Page: https://dikini.github.io/ash/learn/tour/  
Authority: website-guidance  
Source revision: bf8bc187f70efe844d02028d5761f381ca630bff

This tour assumes you have written programs before. New to programming? Start with the [guided learning route](/ash/learn/#new-to-programming).



**The checkpoint matters.** These programs target Ash [bade38c](https://github.com/dikini/ash-lang/tree/bade38c635755a608695dd58c8cbef1254aace4c). The compiler reports a value and its type; these programs do not print text from inside Ash.



## Functions and values



An Ash module contains definitions. The entry module supplies `main`, and the tool evaluates `main ()`. Here, `double` takes an integer and returns an integer.



answer.ash



```ash
double : Int -> Int
double number = number + number

main : Unit -> Int
main () = double 21
```





```text
42 : Int
```





The signature sits above the definition. Function application supplies an argument without requiring parentheses: `double 21`. The entry function takes `Unit`, whose value is `()`.



## Data and patterns



A nominal data type names the possibilities. A reading is either missing or contains an integer. Equations follow those cases directly.



data.ash



```ash
type Reading =
    Missing
    | Measured(Int)

value_or : Int -> Reading -> Int
value_or fallback Missing = fallback
value_or fallback (Measured(value)) = value

main : Unit -> Int
main () = value_or 0 (Measured(42))
```





```text
42 : Int
```





`value_or` receives a fallback and a reading. `Missing` selects the fallback; `Measured` exposes its value. The compiler checks that the patterns cover the possible cases. Try changing `Measured(42)` to `Missing` and predict the new result.



## Effects and handlers



A computation can declare an operation it needs without choosing its interpretation. The effect row `! {Ask}` on `calculate` makes that request visible in its signature.



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





`perform` asks for an answer. The enclosing handler supplies `40` through `resume`, so the calculation continues with `40 + 2`. Changing the handler can change the supplied value while leaving `calculate` alone.



This is a small illustration of lexical handlers, not a concurrency example. Ash checks continuation use; this example resumes it exactly once.



## Try it yourself



[Build the pinned command-line tool](/ash/docs/install/), save one of these complete programs as `main.ash` in the cloned `ash-lang` directory, and run from that directory:



```
./target/debug/ash main.ash
```



In PowerShell, use `.\target\debug\ash.exe main.ash` instead.



Continue with [Study Ash](/ash/study/) for sustained lessons and exercises, or read the [language specification](https://github.com/dikini/ash-lang/blob/bade38c635755a608695dd58c8cbef1254aace4c/spec/README.md) for precise rules. Check [current limitations](/ash/status/) before planning a larger project.
