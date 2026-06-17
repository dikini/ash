-- @test name: quickcheck_canonical_sorted_list
-- @test kind: property
-- @test max_cases: 4
-- @test params: xs: List<Int>, x: Int
-- @test strategy xs: test::quickcheck::list::sorted_ints
-- @test strategy x: test::quickcheck::int::positive
-- @test property: x >= 1

fn main() -> Bool {
    true
}
