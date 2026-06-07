pub mod semigroup;
pub mod monoid;
pub mod functor;
pub mod applicative;
pub mod monad;
pub mod comonad;
pub mod kleisli;

pub use semigroup::{Semigroup};
pub use monoid::{Monoid};
pub use functor::{Functor};
pub use applicative::{Applicative};
pub use monad::{Monad};
pub use comonad::{Comonad};
