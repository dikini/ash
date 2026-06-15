-- @test name: quickcheck_positive_int_override_shrinks_in_domain
-- @test kind: property
-- @test max_cases: 8
-- @test params: x: Int
-- @test strategy x: test::quickcheck::positive_ints
-- @test property: x < 0

fn main() -> Bool {
    true
}
