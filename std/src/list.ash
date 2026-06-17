-- Test with Cons pattern using explicit field patterns
pub fn test(list: List<Int>) -> List<Int> {
    match list {
        Nil => [],
        Cons { head: h, tail: rest } => [h]
    }
}
