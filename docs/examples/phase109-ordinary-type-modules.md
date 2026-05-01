# Phase 109 ordinary type module behavior

This example documents the implemented SPEC-057/TASK-791 closeout behavior for
ordinary type declarations in modules. Ordinary `type` declarations are parsed as
ModuleFile definitions, lowered into semantic summaries, and transported by the
engine/typechecker import path. Source-snippet scanning is not authoritative in
the normal path.

## Library module

```ash
mod model {
    pub type UserId = String;
    pub type Status = Pending | Done;
    type SecretToken = String;

    pub fn default_status() -> Status {
        Pending
    }
}
```

`UserId` and `Status` are public ordinary type exports. Their canonical module
identity is preserved in the semantic summary, so downstream imports refer to the
same type identity rather than reconstructing a local snippet.

`SecretToken` is private. Its name and representation are not public exports and
must not leak through the public summary.

## Importing public ordinary types

```ash
import model::{UserId, Status, default_status};

fn accepts_user(id: UserId) -> Status {
    default_status()
}
```

A downstream module can import public ordinary type names and use them in type
positions. Constructors for exposed public variants, such as `Pending` and
`Done`, are available only through the same representation-visibility rules as
other ordinary type exports.

## Private leak rejection note

A public signature must not expose a private ordinary type such as
`model::SecretToken`. Phase 109 records and checks summary visibility so this is
diagnosed at module-boundary/export time instead of being silently accepted by a
textual type snippet scanner.

Deferred DESIGN-034 features such as type functions, sealed domains,
normalization, associated family computation, and propositions remain outside
Phase 109.
