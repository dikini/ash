-- @test name: quickcheck_default_arbitrary_bool
-- @test kind: property
-- @test max_cases: 2
-- @test params: b: Bool
-- @test property: b == b

use test::quickcheck::{Arbitrary};

fn main() -> Bool {
    true
}
