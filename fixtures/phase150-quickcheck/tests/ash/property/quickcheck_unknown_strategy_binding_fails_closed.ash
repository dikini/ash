-- @test name: quickcheck_unknown_strategy_binding_fails_closed
-- @test kind: property
-- @test max_cases: 4
-- @test params: x: Int
-- @test strategy y: test::quickcheck::positive_ints
-- @test property: x == x

fn main() -> Bool {
    true
}
