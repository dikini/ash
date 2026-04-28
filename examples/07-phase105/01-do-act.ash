// Phase 105 generalized typed do-notation: explicit Act target.
// `let` is ordinary pure binding; `<-` sequences Act computations; final `return`
// wraps through the hidden Act dictionary.

pub fn greeting_action(name: String) -> Act<String> {
    do:Act {
        let prefix = "hello, ";
        message <- act::unit(prefix + name);
        return message
    }
}
