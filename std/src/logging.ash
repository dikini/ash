-- Structured logging provider surface.
--
-- Logging calls are host/report boundary operations. Runtime profiles decide
-- whether a projected application may emit logs, and denied attempts still
-- produce redacted host-boundary evidence.

pub builtin fn debug(message: String) -> Record;
pub builtin fn info(message: String) -> Record;
pub builtin fn warn(message: String) -> Record;
pub builtin fn error(message: String) -> Record;
