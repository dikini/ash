-- @test name: flaky_once
-- @test kind: unit
-- @test flaky_until_attempt: 2
workflow main() -> Bool { ret true }
