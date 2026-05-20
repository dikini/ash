-- OODA library/template compatibility helpers.
--
-- These helpers preserve the historical Observe/Orient/Decide/Act teaching
-- vocabulary as ordinary library calls. They do not introduce primitive IR,
-- AMIR, bytecode, or runtime roots; alpha execution semantics come from the
-- visible tower algebra and explicit Act/Proc/Workflow operations.

-- Marks an observation-shaped value in examples or templates.
pub fn observe<T>(value: T) -> T {
    value
}

-- Marks an orientation-shaped transformation result in examples or templates.
pub fn orient<T>(value: T) -> T {
    value
}

-- Marks a decision-shaped value in examples or templates.
pub fn decide<T>(value: T) -> T {
    value
}

-- Marks an action-shaped value in examples or templates.
pub fn act<T>(value: T) -> T {
    value
}
