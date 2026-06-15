-- @test name: quickcheck_bad_strategy_fails_closed
-- @test kind: property
-- @test max_cases: 4
-- @test params: x: Int
-- @test strategy x: test::quickcheck::sorted_int_lists
-- @test property: x >= 1

fn main() -> Bool {
    true
}
