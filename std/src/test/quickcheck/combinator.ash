use test::quickcheck::context::{GenContext};
use test::quickcheck::strategy::{Strategy};

pub type Weighted<T> = Weighted {
    weight: Int,
    strategy: Strategy<T>,
};

pub type RecursiveConfig = RecursiveConfig {
    max_depth: Int,
    breadth: Int,
};

-- | Map a function over a strategy's generated values.
-- | The resulting strategy has no shrink (use map_with_shrink for that).
pub fn map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B> {
    Strategy {
        gen: fn(ctx) { f(s.gen(ctx)) },
        shrink: fn(_b) { [] }
    }
}

-- | Map with explicit shrink function.
pub fn map_with_shrink<A, B>(s: Strategy<A>, f: (A) -> B, shrink: (B) -> List<B>) -> Strategy<B> {
    Strategy {
        gen: fn(ctx) { f(s.gen(ctx)) },
        shrink: fn(b) { shrink(b) }
    }
}

-- | Map over two strategies.
pub fn map2<A, B, C>(sa: Strategy<A>, sb: Strategy<B>, f: (A, B) -> C) -> Strategy<C> {
    Strategy {
        gen: fn(ctx) { f(sa.gen(ctx), sb.gen(ctx)) },
        shrink: fn(_c) { [] }
    }
}

-- | Wrap a strategy with an explicit shrink function.
pub fn with_shrink<T>(s: Strategy<T>, shrink: (T) -> List<T>) -> Strategy<T> {
    Strategy {
        gen: fn(ctx) { s.gen(ctx) },
        shrink: fn(t) { shrink(t) }
    }
}

-- | Create a strategy that always generates the same value.
pub fn constant<T>(value: T) -> Strategy<T> {
    Strategy {
        gen: fn(_ctx) { value },
        shrink: fn(_t) { [] }
    }
}

-- | Create a weighted choice wrapper.
pub fn weighted<T>(weight: Int, strategy: Strategy<T>) -> Weighted<T> {
    Weighted { weight: weight, strategy: strategy }
}

-- | Choose one strategy from a list uniformly at random.
pub fn one_of<T>(strategies: List<Strategy<T>>) -> Strategy<T> {
    Strategy {
        gen: fn(ctx) {
            let index = choose_int(ctx, 0, len(strategies) - 1)
            index_strategies(strategies, index).gen(ctx)
        },
        shrink: fn(_t) { [] }
    }
}

-- | Choose one strategy from a list of weighted choices.
-- Uses cumulative weights and binary search via recursive helper.
pub fn one_of_weighted<T>(choices: List<Weighted<T>>) -> Strategy<T> {
    Strategy {
        gen: fn(ctx) {
            let total = sum_weights(choices, 0)
            let target = choose_int(ctx, 1, total)
            pick_weighted(choices, target, 0).gen(ctx)
        },
        shrink: fn(_t) { [] }
    }
}

-- | Append extra shrink candidates to a strategy's shrink list.
pub fn append_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T> {
    Strategy {
        gen: fn(ctx) { s.gen(ctx) },
        shrink: fn(t) { concat(s.shrink(t), extra) }
    }
}

-- | Prepend extra shrink candidates to a strategy's shrink list.
pub fn prepend_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T> {
    Strategy {
        gen: fn(ctx) { s.gen(ctx) },
        shrink: fn(t) { concat(extra, s.shrink(t)) }
    }
}

-- | Helper: sum all weights in a list.
fn sum_weights<T>(choices: List<Weighted<T>>, acc: Int) -> Int {
    match choices {
        [] => acc,
        [w, ..rest] => sum_weights(rest, acc + w.weight)
    }
}

-- | Helper: pick the strategy whose cumulative weight reaches target.
fn pick_weighted<T>(choices: List<Weighted<T>>, target: Int, acc: Int) -> Strategy<T> {
    match choices {
        [] => panic "pick_weighted: target exceeded total weight",
        [w, ..rest] => {
            let new_acc = acc + w.weight
            match new_acc >= target {
                true => w.strategy,
                false => pick_weighted(rest, target, new_acc)
            }
        }
    }
}

-- | Helper: index into a list of strategies (0-based).
fn index_strategies<T>(strategies: List<Strategy<T>>, index: Int) -> Strategy<T> {
    match strategies {
        [] => panic "index_strategies: index out of bounds",
        [s, ..rest] => match index == 0 {
            true => s,
            false => index_strategies(rest, index - 1)
        }
    }
}

-- | Default recursive config: max_depth=5, breadth=3.
pub fn default_recursive_config() -> RecursiveConfig {
    RecursiveConfig { max_depth: 5, breadth: 3 }
}

-- | Create a recursive config with explicit parameters.
pub fn recursive_config(max_depth: Int, breadth: Int) -> RecursiveConfig {
    RecursiveConfig { max_depth: max_depth, breadth: breadth }
}

