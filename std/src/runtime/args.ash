-- Runtime-provided command-line arguments capability

use option::Option;

pub capability Args: observe(index: Int) returns Option<String>;
