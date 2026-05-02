pub capability interface KeyValue:
    observe get(key: String) returns String
  | execute put(key: String, value: String) returns Unit;

pub resource type CacheKV {
    namespace: String
}

pub capability impl MockInternalKV for KeyValue
    requires resource store: CacheKV
    requires config fixture: String
{
    observe get(key: String) returns String { fixture }
    execute put(key: String, value: String) returns Unit { null }
}

pub capability impl LoggingCacheKV for KeyValue
    requires resource cache: CacheKV
    requires capability inner: KeyValue
    requires config prefix: String
{
    observe get(key: String) returns String { key }
    execute put(key: String, value: String) returns Unit { null }
}
