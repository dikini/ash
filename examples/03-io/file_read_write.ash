// File Read/Write Example
//
// Real file reads and writes require configured Fs/Stdio capabilities. This
// checkable example keeps the file names and payload explicit without claiming
// to touch the host filesystem.

workflow main {
    let input_path = "input.txt"
    let output_path = "output.txt"
    let payload = "Hello, World!"

    ret payload
}
