pub capability interface KeyValue:
    observe get(key: String) returns String
  | execute put(key: String, value: String) returns Unit;

pub resource type WorkflowKV {
    namespace: String
}

pub capability impl MockInternalKV for KeyValue
    requires resource store: WorkflowKV
    requires config fixture: String
{
    observe get(key: String) returns String { fixture }
    execute put(key: String, value: String) returns Unit { () }
}
