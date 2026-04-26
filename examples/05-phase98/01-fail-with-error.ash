workflow main {
    ret with_error { fail "boom" } handle { _ => 7; }
}
