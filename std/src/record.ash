-- Record builtins: keys, values, and the record constructor.
--
-- `keys` returns a List<String> (record keys are always strings).
-- `values` returns a heterogeneous list; List<String> below is a placeholder
-- pending polymorphic type support (Track D1.5/D2).
-- `record` is variadic (accepts key-value pairs) but is declared with 0
-- parameters here; the dispatch table entry has variadic: true.
pub builtin fn keys(r: Record) -> List<String>;
-- placeholder return type: runtime returns List<Any> (values are heterogeneous)
pub builtin fn values(r: Record) -> List<String>;
pub builtin fn record() -> Record;
