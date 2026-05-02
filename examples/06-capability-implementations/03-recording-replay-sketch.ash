pub capability interface KeyValue:
    observe get(key: String) returns String
  | execute put(key: String, value: String) returns Unit;

pub resource type ReplayLog {
    namespace: String
}

pub capability impl RecordingKV for KeyValue
    requires resource log: ReplayLog
    requires capability inner: KeyValue
    requires config label: String
{
    observe get(key: String) returns String { label }
    execute put(key: String, value: String) returns Unit { null }
}

pub capability impl ReplayKV for KeyValue
    requires resource log: ReplayLog
    requires config recorded: String
{
    observe get(key: String) returns String { recorded }
    execute put(key: String, value: String) returns Unit { null }
}
