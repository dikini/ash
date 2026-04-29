// Path Operations Example
//
// Pure path helper calls are covered by the stdlib module corpus. This example
// keeps path manipulation checkable with currently supported string syntax.

workflow main {
    let root_path = "/tmp"
    let file_name = "example.txt"
    let file_path = "/tmp/example.txt"

    ret file_path
}
