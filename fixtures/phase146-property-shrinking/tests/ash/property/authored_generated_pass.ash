-- @test name: authored_generated_reflexive
-- @test kind: property
-- @test params: x: Int, xs: List<Int>, opt: Option<Int>
-- @test property: x == x
workflow main() -> Bool { ret true }
