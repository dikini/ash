-- Record builtins: keys, values, and the record constructor.
--
-- At runtime, `keys` and `values` accept a Record and return a List<String>.
-- `record` is variadic (accepts key-value pairs) but is declared with 0
-- parameters here; the dispatch table entry has variadic: true.
pub builtin fn keys(r: Record) -> List<String>;
pub builtin fn values(r: Record) -> List<String>;
pub builtin fn record() -> Record;
