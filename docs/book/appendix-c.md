# Appendix C: File Structure

This appendix records the current productive documentation and example roots after Phase 201
deprecated-form removal. Older mdBook chapter maps and workflow example trees were removed or
quarantined because they described deleted source forms as current behavior.

## Productive Documentation

```text
docs/
├── README.md
├── TUTORIAL.md
├── API.md
├── tutorials/
│   └── phase199-productive-apps.md
├── book/
│   ├── SUMMARY.md
│   ├── appendix-a.md
│   ├── appendix-b.md
│   └── appendix-c.md
├── reference/
├── spec/
├── notes/
└── plan/
```

## Productive Examples

```text
examples/
├── README.md
├── 10-testing-helpers/
│   └── testing_helpers.ash
└── 11-process-channel-helpers/
    └── process_channel_helpers.ash
```

Run the checked examples from the repository root:

```bash
ash check examples/10-testing-helpers/testing_helpers.ash
ash check examples/11-process-channel-helpers/process_channel_helpers.ash
```

Historical/reference documents may still discuss older design eras as prose. They are not
productive Ash source, examples, templates, or fixtures unless a current Phase 201 gate identifies
them as such.
