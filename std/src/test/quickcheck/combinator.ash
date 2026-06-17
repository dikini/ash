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

pub builtin fn one_of<T>(choices: List<Strategy<T>>) -> Strategy<T>;
pub builtin fn one_of_weighted<T>(choices: List<Weighted<T>>) -> Strategy<T>;
pub builtin fn map<A, B>(s: Strategy<A>, f: (A) -> B) -> Strategy<B>;
pub builtin fn map_with_shrink<A, B>(s: Strategy<A>, f: (A) -> B, shrink: (B) -> List<B>) -> Strategy<B>;
pub builtin fn map2<A, B, C>(sa: Strategy<A>, sb: Strategy<B>, f: (A, B) -> C) -> Strategy<C>;
pub builtin fn with_shrink<T>(s: Strategy<T>, shrink: (T) -> List<T>) -> Strategy<T>;
pub builtin fn append_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>;
pub builtin fn prepend_shrink<T>(s: Strategy<T>, extra: List<T>) -> Strategy<T>;
pub builtin fn recursive<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, config: RecursiveConfig) -> Strategy<T>;
pub builtin fn recursive_with<T>(base: Strategy<T>, rec: (Strategy<T>) -> Strategy<T>, max_depth: Int, breadth: Int) -> Strategy<T>;

pub fn weighted<T>(weight: Int, strategy: Strategy<T>) -> Weighted<T> {
    Weighted { weight: weight, strategy: strategy }
}

pub fn default_recursive_config() -> RecursiveConfig {
    RecursiveConfig { max_depth: 5, breadth: 3 }
}

pub fn recursive_config(max_depth: Int, breadth: Int) -> RecursiveConfig {
    RecursiveConfig { max_depth: max_depth, breadth: breadth }
}
