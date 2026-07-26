use vstd::prelude::*;

verus! {
    proof fn one_is_one()
        ensures 1int == 1int,
    {
    }
}
