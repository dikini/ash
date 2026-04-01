-- Runtime error type for entry-point workflows

pub type RuntimeError = RuntimeError {
    exit_code: Int,
    message: String
};
