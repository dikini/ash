// Directory Operations Example
//
// Directory creation/listing/removal require runtime Dir/Meta capabilities.
// This checkable example names planned paths without performing host IO.

workflow main {
    let project_path = "/tmp/test_ash"
    let nested_path = "/tmp/test_ash/nested/deep"
    ret project_path
}
