-- Result type and helper functions
--
-- Result<T, E> represents either success (Ok) or failure (Err).

pub type Result<T, E> = Ok { value: T } | Err { error: E };

-- Returns true if the result is Ok
pub fn is_ok<T, E>(res: Result<T, E>) -> Bool {
    match res {
        Ok { value: _ } => true,
        Err { error: _ } => false
    }
}

-- Returns true if the result is Err
pub fn is_err<T, E>(res: Result<T, E>) -> Bool {
    match res {
        Ok { value: _ } => false,
        Err { error: _ } => true
    }
}

-- Returns the value if Ok, otherwise panics
pub fn unwrap<T, E>(res: Result<T, E>) -> T {
    match res {
        Ok { value: v } => v,
        Err { error: _ } => panic "called `unwrap` on an `Err` value"
    }
}

-- Returns the error if Err, otherwise panics
pub fn unwrap_err<T, E>(res: Result<T, E>) -> E {
    match res {
        Ok { value: _ } => panic "called `unwrap_err` on an `Ok` value",
        Err { error: e } => e
    }
}

-- Returns the value if Ok, otherwise returns default
pub fn unwrap_or<T, E>(res: Result<T, E>, default: T) -> T {
    match res {
        Ok { value: v } => v,
        Err { error: _ } => default
    }
}

-- Maps Result<T, E> to Result<U, E> by applying a function to Ok
pub fn map<T, E, U>(res: Result<T, E>, f: Fun(T) -> U) -> Result<U, E> {
    match res {
        Ok { value: v } => Ok { value: f(v) },
        Err { error: e } => Err { error: e }
    }
}

-- Maps Result<T, E> to Result<T, F> by applying a function to Err
pub fn map_err<T, E, F>(res: Result<T, E>, f: Fun(E) -> F) -> Result<T, F> {
    match res {
        Ok { value: v } => Ok { value: v },
        Err { error: e } => Err { error: f(e) }
    }
}

-- Chains operations that return Result
pub fn and_then<T, E, U>(res: Result<T, E>, f: Fun(T) -> Result<U, E>) -> Result<U, E> {
    match res {
        Ok { value: v } => f(v),
        Err { error: e } => Err { error: e }
    }
}

-- Converts Result<T, E> to Option<T>
pub fn ok<T, E>(res: Result<T, E>) -> Option<T> {
    match res {
        Ok { value: v } => Some { value: v },
        Err { error: _ } => None
    }
}

-- Converts Result<T, E> to Option<E>
pub fn err<T, E>(res: Result<T, E>) -> Option<E> {
    match res {
        Ok { value: _ } => None,
        Err { error: e } => Some { value: e }
    }
}
