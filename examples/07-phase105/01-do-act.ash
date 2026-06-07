// Phase 105 generalized typed do-notation: explicit Act target.
// `let` is ordinary pure binding; `<-` sequences Act computations; final `return`
// wraps through Phase-133 selected public/named `Monad<Act>` evidence; the old anonymous bridge wording is historical and superseded.

pub fn greeting_action(name: String) -> Act<String> {
    do:Act {
        let prefix = "hello, ";
        message <- act::unit(prefix + name);
        return message
    }
}
