# Ash-doc integration

Date: 2026-10-17

Status: ownership model agreed; artifact integration details proposed.

## Different contexts under one public home

Ash-site is informational and changes with the project. Ash-doc is reference
and study material whose explanations, examples, observations, and checks form
a coherent edition. The website can use its methods, link to it, or host its
outputs without requiring both projects to change at the same pace.

Keep ash-doc separately maintained initially. If the repositories are later
consolidated, retain a distinct study source tree, its verification boundary,
and its independent edition build. A repository move must preserve history,
canonical examples, fixtures, checks, source relationships, and book outputs.
Consolidation is not required for the first website version.

## Ownership

| Material | Owner | Website treatment |
| --- | --- | --- |
| Public overview, short tour, practical guides, status, updates | ash-site | Author and review as website content |
| Sequential lessons, exercises, worked cases, study introductions | ash-doc | Link to or publish its generated edition |
| Study programs, diagnostic fixtures, and executable checks | ash-doc | Preserve with canonical study source |
| Study HTML, EPUB, book PDF, and A4 PDF | ash-doc | Distribute approved outputs without reinterpreting source |
| Language specification and implementation | ash-lang | Link to authoritative, appropriately versioned material |
| Website example programs and their checks | ash-site | Verify against an explicit language checkpoint |

Study content may inspire a shorter website explanation. If adapted, review
its source, attribution, license, and target. The shorter explanation acquires
its own stated scope; it does not become a second owner of the original lesson.

## Existing baseline

The local ash-doc README inspected on 2026-10-17 declares `diag-ash` at
`9ec5920e8c395f9a36125805ee93b7f58c0f4764` as its language target. It reports 54
verified lessons across Learn, Libraries, Effects, and Cases. The documentation
is unpublished; its full rendering integration is not yet complete for the
latest added Case. These are recorded upstream declarations, not verification
performed by ash-site.

Ash-doc currently uses Quarto Book and Typst for HTML, EPUB 3, a book PDF, and
an A4 PDF. It retains canonical code, fixtures, lesson checks, rendering checks,
and a smaller regression specimen. Preserve these capabilities.

Read the current upstream records before preparing a publication; this snapshot
does not freeze future ash-doc development:

- [Ash-doc repository](https://github.com/dikini/ash-doc)
- [Rendering guide](https://github.com/dikini/ash-doc/blob/main/docs/project/rendering.md)
- [Verification guide](https://github.com/dikini/ash-doc/blob/main/docs/project/verification.md)

## Publication interface

Publish the complete approved static HTML output under
`/study/<edition>/`, with links to EPUB and both PDF variants. Keep book
navigation, chapter order, notes, figures, code, and diagnostics intact. Use
ordinary navigation into the study publication rather than an iframe or
rendering its Markdown a second time through the website engine.

Each import must identify:

- the edition identifier and immutable ash-doc source revision;
- the exact Ash target revision and milestone;
- the approved artifact location, file hashes, and verification record;
- the publication state and available download formats.

Record that identity in a small manifest when implementing the first import.
Do not invent an artifact service or generalized registry beforehand. Import
explicitly selected artifacts, never whichever build happens to be newest.
Published editions retain their URLs and bytes; changes require a new edition
identifier or the publication policy's explicit correction procedure.

The stable `/study/` page lists available editions and their language targets.
The normal website build must work without a sibling ash-doc checkout or an
Ash compiler. Missing explicitly requested publication artifacts must fail the
publication assembly rather than produce broken navigation.

## Integration checks

Before accepting the first embedded edition:

1. Verify upstream publication approval, provenance, redistribution permissions,
   and the edition's required executable and rendering evidence.
2. Serve the candidate under its final nested path. Check chapter navigation,
   anchors, images, scripts, styles, fonts, and downloads for path assumptions.
3. Compare imported artifact hashes with the approved publication. If the
   renderer needs path changes, generate and verify a new upstream artifact;
   avoid unrecorded post-processing of publication files.
4. Check representative introductory and advanced pages on narrow and wide
   screens, including notes, long code, diagnostics, and figures.
5. Confirm that edition targets remain visible and that moving website
   navigation cannot make an old edition appear current.
6. Keep website search separate initially. Verify the study publication's own
   search under its nested path if supplied; add cross-edition search only
   after edition labels and filtering are designed.

Upstream owns executing study examples and producing book evidence. Ash-site
owns artifact integrity, hosting paths, navigation, and download availability.
A status-text change should not rebuild or retest an unchanged book. A new
compiler target must not reuse evidence from another target.

If no publication is ready, ship a study landing page with honest source links.
Do not present local review outputs as a released edition.
