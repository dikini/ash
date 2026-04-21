-- HTTP client capability and functions
--
-- Provides functions for making HTTP requests.
-- All functions require the Http capability.

-- Http capability for HTTP client operations
pub capability Http: execute get(url: String) returns String
                   | execute post(url: String, body: String) returns String
                   | execute put(url: String, body: String) returns String
                   | execute delete(url: String) returns String
                   | observe head(url: String) returns String;

-- Perform an HTTP GET request
pub fn get(url: String) -> String {
    act execute Http.get with url: url
}

-- Perform an HTTP POST request with a body
pub fn post(url: String, body: String) -> String {
    act execute Http.post with url: url, body: body;
}

-- Perform an HTTP PUT request with a body
pub fn put(url: String, body: String) -> String {
    act execute Http.put with url: url, body: body;
}

-- Perform an HTTP DELETE request
pub fn delete(url: String) -> String {
    act execute Http.delete with url: url;
}
