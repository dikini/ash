-- @test name: quickcheck_source_seed_cli_override
-- @test kind: property
-- @test seed: 7
-- @test max_cases: 2
-- @test params: x: Int
-- @test strategy x: test::quickcheck::int::positive
-- @test property: x >= 1

fn main() -> Bool {
    true
}
