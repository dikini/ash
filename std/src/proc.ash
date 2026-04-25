-- Proc library helpers
--
-- Phase 98 library surface for process-structured computations.
-- This initial slice exposes only non-concurrent construction/sequencing.
-- Child process creation, process handles, observation, and scheduler yield are
-- owned by later PLAN-098 tasks.

pub builtin fn unit<A>(v: A) -> Proc<A>;
pub builtin fn bind<A, B>(ma: Proc<A>, f: A -> Proc<B>) -> Proc<B>;
pub builtin fn then<A, B>(ma: Proc<A>, mb: Proc<B>) -> Proc<B>;
