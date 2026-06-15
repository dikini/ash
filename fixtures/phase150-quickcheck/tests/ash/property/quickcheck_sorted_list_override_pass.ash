-- @test name: quickcheck_sorted_list_override_metadata
-- @test kind: property
-- @test max_cases: 4
-- @test params: xs: List<Int>, x: Int
-- @test strategy xs: test::quickcheck::sorted_int_lists
-- @test strategy x: test::quickcheck::positive_ints
-- @test property: x >= 1

fn main() -> Bool {
    true
}
