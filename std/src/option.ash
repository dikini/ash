-- Option type and helper functions
--
-- Option<T> represents an optional value: either Some(T) or None.

pub type Option<T> = Some { value: T } | None;

-- Returns true if the option is Some
pub fn is_some<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => true,
        None => false
    }
}

-- Returns true if the option is None
pub fn is_none<T>(opt: Option<T>) -> Bool {
    match opt {
        Some { value: _ } => false,
        None => true
    }
}

-- Returns the value if Some, otherwise panics
pub fn unwrap<T>(opt: Option<T>) -> T {
    match opt {
        Some { value: v } => v,
        None => panic "called `unwrap` on a `None` value"
    }
}

-- Returns the value if Some, otherwise returns default
pub fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some { value: v } => v,
        None => default
    }
}

-- Maps Option<T> to Option<U> by applying a function
pub fn map<T, U>(opt: Option<T>, f: Fun(T) -> U) -> Option<U> {
    match opt {
        Some { value: v } => Some { value: f(v) },
        None => None
    }
}

-- Returns None if the option is None, otherwise returns optb
pub fn and<T>(opt: Option<T>, optb: Option<T>) -> Option<T> {
    match opt {
        Some { value: _ } => optb,
        None => None
    }
}

-- Returns the option if it contains a value, otherwise returns optb
pub fn or<T>(opt: Option<T>, optb: Option<T>) -> Option<T> {
    match opt {
        Some { value: _ } => opt,
        None => optb
    }
}

-- Converts Option<T> to Result<T, E>
pub fn ok_or<T, E>(opt: Option<T>, err: E) -> Result<T, E> {
    match opt {
        Some { value: v } => Ok { value: v },
        None => Err { error: err }
    }
}
