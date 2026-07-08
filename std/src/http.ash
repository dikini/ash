-- HTTP client capability and functions
--
-- Provides runtime-backed function declarations for making HTTP requests. The
-- Http capability records the intended effectful authority contract; Phase 198
-- profiles admit explicit HTTP rows and enforce host/method sandbox policy plus
-- redacted provenance at the projected provider boundary.

-- HTTP provider operations return runtime records with at least:
--   status: Int, headers: Record, body: String
-- HEAD remains deferred: the runtime provider returns status/header metadata for
-- HEAD, while the existing unqualified `head` builtin belongs to lists. Exposing
-- `http::head` as a plain builtin would currently collide with list dispatch.
-- Perform an HTTP GET request
pub builtin fn get(url: String) -> Record;

-- Perform an HTTP POST request with a body
pub builtin fn post(url: String, body: String) -> Record;

-- Perform an HTTP PUT request with a body
pub builtin fn put(url: String, body: String) -> Record;

-- Perform an HTTP DELETE request
pub builtin fn delete(url: String) -> Record;
