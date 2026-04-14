-- Map type (key-value association list)
--
-- Map<K, V> is an association list: a flat list of key-value pairs.
-- Provides O(n) lookup by key, suitable for small maps and template substitution.
-- For large maps, a future hash-map implementation will be more efficient.

pub type Map<K, V> = List<(K, V)>;
