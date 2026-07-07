-- Structured logging provider surface.
--
-- Logging calls are host/report boundary operations. Runtime profiles decide
-- whether a projected application may emit logs, and denied attempts still
-- produce redacted host-boundary evidence.

pub capability Logging: execute debug(message: String)
                      | execute info(message: String)
                      | execute warn(message: String)
                      | execute error(message: String);

pub builtin fn debug(message: String) -> Unit;
pub builtin fn info(message: String) -> Unit;
pub builtin fn warn(message: String) -> Unit;
pub builtin fn error(message: String) -> Unit;
