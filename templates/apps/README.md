# Canonical App Templates

Phase 199 canonical app templates live in this directory. Each subdirectory contains an
`ash-template-v1` `template.json` manifest and is validated through `ash template instantiate` in
the `phase199_canonical_templates` CLI test.

## Templates

- `cli-tool`: CLI tool skeleton over the application-default profile.
- `file-pipeline`: filesystem read/write pipeline skeleton.
- `http-fetch-process`: sandboxed HTTP fetch/process skeleton.
- `supervised-worker`: process/logging worker skeleton with trace expectations.
- `provider-profile-test`: deterministic provider/profile test app skeleton.
