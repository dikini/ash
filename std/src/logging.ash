-- Structured logging provider surface.
--
-- Logging calls are host/report boundary operations. Runtime profiles decide
-- whether a projected application may emit logs, and denied attempts still
-- produce redacted host-boundary evidence.

pub builtin fn debug(message: String) -> { level: String, redacted: String, field_count: Int };
pub builtin fn info(message: String) -> { level: String, redacted: String, field_count: Int };
pub builtin fn warn(message: String) -> { level: String, redacted: String, field_count: Int };
pub builtin fn error(message: String) -> { level: String, redacted: String, field_count: Int };
