effect Ask {
    answer : Unit -> Int
}

calculate : Unit -> Int ! {Ask}
calculate () = (perform Ask.answer ()) + 2

main : Unit -> Int
main () = handle (calculate ()) with Ask {
    answer () => resume 40
}
