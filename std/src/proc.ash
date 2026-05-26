-- Proc library helpers
--
-- Phase 98 library surface for process-structured computations.
-- This slice includes single-handle observation via `proc::await`,
-- cooperative scheduler yield via `proc::yield`, and all-or-none child
-- admission via `proc::par` / `proc::scatter`.
-- Wait-for-all observation includes `proc::join` / `proc::gather`.

pub type ParHandles<A, B> = (P<A>, P<B>);

pub builtin fn unit<A>(v: A) -> Proc<A>;
pub builtin fn from_act<A>(ma: Act<A>) -> Proc<A>;
pub builtin fn bind<A, B>(ma: Proc<A>, f: (A) -> Proc<B>) -> Proc<B>;
pub builtin fn then<A, B>(ma: Proc<A>, mb: Proc<B>) -> Proc<B>;
pub builtin fn await<A>(handle: P<A>) -> Proc<A>;
pub builtin fn yield() -> Proc<Unit>;
pub builtin fn par<A, B>(left: Proc<A>, right: Proc<B>) -> Proc<ParHandles<A, B>>;
pub builtin fn scatter<A, B>(items: List<A>, f: (A) -> Proc<B>) -> Proc<List<P<B>>>;
pub builtin fn join<A, B>(left: P<A>, right: P<B>) -> Proc<(A, B)>;
pub builtin fn gather<A>(handles: List<P<A>>) -> Proc<List<A>>;
