# Phase 199 App Template Manifest Schema

Phase 199 app templates use the `ash-template-v1` manifest schema. Manifests are metadata only:
they describe generated files, required profiles, provider expectations, parameters, and checks.
They do not grant authority and must validate before any instantiation step.

## Required Fields

- `schema_version`: must be `ash-template-v1`.
- `id`: stable lowercase identifier using ASCII letters, digits, `.`, `_`, or `-`.
- `version`: numeric `major.minor.patch`.
- `description`: non-empty human-facing summary.
- `required_profiles`: explicit profile names required by the template.
- `providers`: provider expectations scoped to declared profiles.
- `resources`: explicit resource expectations used by generated files.
- `evidence_expectations`: evidence or checks expected from generated output.
- `parameters`: template parameters.
- `files`: relative generated files and current-syntax contents.
- `generated_checks`: commands and target files that validate generated output.

## Validation Rules

- Empty, duplicate, stale, unsafe, or undeclared metadata fails closed.
- Provider expectations must reference a declared profile and explicit operations.
- File paths must be relative and must not contain parent traversal, drive prefixes, or absolute
  roots.
- Generated checks must reference declared generated files.
- Migration note: productive template files reject stale syntax patterns such as migration-only
  observe-with and act-with forms, plus historical tower-carrier type spellings.

## Instantiation

`ash template instantiate --manifest <path> --out <dir> --param key=value` loads a JSON manifest,
validates it, substitutes only declared parameters using `{{name}}` placeholders, writes generated
files, and runs declared `ash check` commands. Existing files are protected unless `--overwrite` is
provided.
