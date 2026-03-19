# Ash Standard Library

The Ash Standard Library provides core types and helper functions for the Ash workflow language.

## Structure

```
std/
├── src/
│   ├── lib.ash      -- Main library exports
│   ├── prelude.ash  -- Automatically imported in all modules
│   ├── option.ash   -- Option<T> type and helpers
│   └── result.ash   -- Result<T, E> type and helpers
```

## Modules

### Option

The `Option<T>` type represents an optional value: either `Some(T)` or `None`.

```ash
pub type Option<T> =
    | Some { value: T }
    | None;
```

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `is_some` | `Option<T> -> Bool` | Returns true if the option is Some |
| `is_none` | `Option<T> -> Bool` | Returns true if the option is None |
| `unwrap` | `Option<T> -> T` | Returns the value if Some, otherwise panics |
| `unwrap_or` | `(Option<T>, T) -> T` | Returns the value if Some, otherwise returns default |
| `map` | `(Option<T>, Fun(T) -> U) -> Option<U>` | Maps Option<T> to Option<U> |
| `and` | `(Option<T>, Option<T>) -> Option<T>` | Returns optb if Some, otherwise None |
| `or` | `(Option<T>, Option<T>) -> Option<T>` | Returns self if Some, otherwise optb |
| `ok_or` | `(Option<T>, E) -> Result<T, E>` | Converts Option to Result |

### Result

The `Result<T, E>` type represents either success (`Ok`) or failure (`Err`).

```ash
pub type Result<T, E> =
    | Ok { value: T }
    | Err { error: E };
```

#### Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `is_ok` | `Result<T, E> -> Bool` | Returns true if the result is Ok |
| `is_err` | `Result<T, E> -> Bool` | Returns true if the result is Err |
| `unwrap` | `Result<T, E> -> T` | Returns the value if Ok, otherwise panics |
| `unwrap_err` | `Result<T, E> -> E` | Returns the error if Err, otherwise panics |
| `unwrap_or` | `(Result<T, E>, T) -> T` | Returns the value if Ok, otherwise default |
| `map` | `(Result<T, E>, Fun(T) -> U) -> Result<U, E>` | Maps Ok value |
| `map_err` | `(Result<T, E>, Fun(E) -> F) -> Result<T, F>` | Maps Err value |
| `and_then` | `(Result<T, E>, Fun(T) -> Result<U, E>) -> Result<U, E>` | Chains operations |
| `ok` | `Result<T, E> -> Option<T>` | Converts Result to Option |
| `err` | `Result<T, E> -> Option<E>` | Converts Result to Option of error |

## Prelude

The prelude is automatically imported in all Ash modules and includes:

- `Option`, `Some`, `None`
- `Result`, `Ok`, `Err`
- Common helper functions: `is_some`, `is_none`, `is_ok`, `is_err`, `unwrap`, `unwrap_or`

## Usage

```ash
-- Option usage
let maybe_value: Option<Int> = Some { value: 42 };
if is_some(maybe_value) then {
    let value = unwrap(maybe_value);
    act log with "Value is: " ++ value;
};

-- Result usage  
let result: Result<Int, String> = Ok { value: 42 };
if is_ok(result) then {
    act log with "Success!";
};
```
