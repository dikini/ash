use vstd::prelude::*;

verus! {
    proof fn one_is_two()
        ensures 1int == 2int,
    {
    }
}
