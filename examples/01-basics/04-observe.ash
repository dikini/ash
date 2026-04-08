workflow main {
    observe some_capability as data
    let temperature = 75
    if temperature > 80 then observe alert_ops as alert else observe log_reading as log
    ret data
}
