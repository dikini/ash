-- Workflow algebra helpers
--
-- Phase 122 public surface for governed process computations.
-- The value-level algebra is source-visible here. Contract operations such as
-- `workflow::requires` and `workflow::ensures` remain compiler-prelude metadata
-- because their parameter classes are not source-denotable Ash types yet.

pub builtin fn unit<A>(v: A) -> Workflow<A>;
pub builtin fn bind<A, B>(ma: Workflow<A>, f: (A) -> Workflow<B>) -> Workflow<B>;
pub builtin fn then<A, B>(ma: Workflow<A>, mb: Workflow<B>) -> Workflow<B>;
pub builtin fn from_proc<A>(ma: Proc<A>) -> Workflow<A>;
pub builtin fn from_act<A>(ma: Act<A>) -> Workflow<A>;
