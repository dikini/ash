-- @test name: quickcheck_duplicate_strategy_binding_fails_closed
-- @test kind: property
-- @test max_cases: 4
-- @test params: x: Int
-- @test strategy x: test::quickcheck::positive_ints
-- @test strategy x: test::quickcheck::small_ints
-- @test property: x == x

fn main() -> Bool {
    true
}
