-- HTTP client capability and functions
--
-- Provides parser-checkable runtime-provided function declarations for making
-- HTTP requests. The Http capability below records the intended effectful
-- authority contract; concrete capability-wrapper bodies remain deferred until
-- the parser/runtime support a canonical stdlib `act` wrapper spelling.

-- HTTP provider operations return runtime records with at least:
--   status: Int, headers: Record, body: String
-- HEAD remains deferred: the runtime provider returns status/header metadata for
-- HEAD, while the existing unqualified `head` builtin belongs to lists. Exposing
-- `http::head` as a plain builtin would currently collide with list dispatch.
pub capability Http: execute get(url: String) returns Record
                   | execute post(url: String, body: String) returns Record
                   | execute put(url: String, body: String) returns Record
                   | execute delete(url: String) returns Record;

-- Perform an HTTP GET request
pub builtin fn get(url: String) -> Record;

-- Perform an HTTP POST request with a body
pub builtin fn post(url: String, body: String) -> Record;

-- Perform an HTTP PUT request with a body
pub builtin fn put(url: String, body: String) -> Record;

-- Perform an HTTP DELETE request
pub builtin fn delete(url: String) -> Record;
