-- QuickCheck v1 generator context surface.
--
-- Generators should use helper functions rather than inspecting fields directly.

pub type GenContext = GenContext {
    seed: Int,
    size: Int,
};

pub builtin fn seed(ctx: GenContext) -> Int;
pub builtin fn size(ctx: GenContext) -> Int;
pub builtin fn split(ctx: GenContext, branch: Int) -> GenContext;
pub builtin fn variant(ctx: GenContext, name: String) -> GenContext;
pub builtin fn indexed(ctx: GenContext, name: String, index: Int) -> GenContext;
pub builtin fn resize(ctx: GenContext, new_size: Int) -> GenContext;
pub builtin fn choose_int(ctx: GenContext, min: Int, max: Int) -> Int;
pub builtin fn choose_bool(ctx: GenContext) -> Bool;
