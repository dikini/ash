# SPEC-008: Retired Dynamic Policy Registration

This specification is retired. Ash does not support dynamic policy registration or dynamic module
loading. Static file-backed and inline module acquisition remain separate, bounded source
acquisition paths; package, registry, hot-reload, and runtime module loading are not provided.

See TASK-2077 and the current module realization plan for the supported boundary.
