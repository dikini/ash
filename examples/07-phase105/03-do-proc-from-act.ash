// Proc do-notation uses Proc sequencing. Act work must cross the tower through
// the explicit proc::from_act boundary; raw Act<T> does not bind in do:Proc.

pub fn proc_greeting(name: String) -> Proc<String> {
    do:Proc {
        message <- proc::from_act(do:Act {
            value <- act::unit("hello, " + name);
            return value
        });
        return message
    }
}
