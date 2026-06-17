-- @test name: quickcheck_canonical_positive_source_cases
-- @test kind: property
-- @test max_cases: 2
-- @test params: x: Int
-- @test strategy x: test::quickcheck::int::positive
-- @test property: x >= 1

fn main() -> Bool {
    true
}
