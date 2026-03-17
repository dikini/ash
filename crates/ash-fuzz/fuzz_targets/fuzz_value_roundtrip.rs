#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz value serialization roundtrip
    use ash_core::value::Value;
    
    // Try to deserialize from bytes (may fail for invalid input)
    if let Ok(value) = serde_json::from_slice::<Value>(data) {
        // Roundtrip: serialize and deserialize should produce equal value
        let serialized = serde_json::to_vec(&value).unwrap();
        let deserialized: Value = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }
});
