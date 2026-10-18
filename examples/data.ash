type Reading =
    Missing
    | Measured(Int)

value_or : Int -> Reading -> Int
value_or fallback Missing = fallback
value_or fallback (Measured(value)) = value

main : Unit -> Int
main () = value_or 0 (Measured(42))
