// New-form expression-level `act { ... }` is sugar for `do:Act { ... }`.
// Legacy `ret` is accepted only for migration; prefer final `return`.

pub fn greeting_action_sugar(name: String) -> Act<String> {
    act {
        let prefix = "hello, ";
        message <- act::unit(prefix + name);
        return message
    }
}
