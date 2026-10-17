# Purpose, audiences, and content structure

Date: 2026-10-17

Status: purpose, audience priorities, and ownership model agreed. Navigation
and first-version scope below are proposed within that model.

## Purpose

Ash-site is Ash's public home. It supports education, marketing, language
status reporting, development updates, documentation, and participation.
Visitors should be able to answer:

1. What is Ash, and why might I want to explore it?
2. What does an Ash program look like?
3. What can I do with the current language and tools?
4. How do I begin, find a precise answer, or study a topic deeply?
5. What is changing, and how can I contribute?

Marketing should explain concrete properties through representative programs
and supported claims. Label aspirations as aspirations. State limitations
where they affect a visitor's evaluation.

## Audiences

| Audience | Main questions | Intended route |
| --- | --- | --- |
| Experienced programmers, primary | Why Ash? What transfers from languages I know? What is usable now? | Home, Discover, quick tour, status, practical documentation |
| Language researchers | What are the semantics, reasoning goals, evidence, and unresolved questions? | Language overview, specification links, design rationale, experimental status |
| Contributors | How is the project organized? What work is useful? How do I build and verify changes? | Contribute, current upstream workflow, source and issue links |
| Novice programmers | How do I set up, understand a first program, practice, and continue? | Learn, setup guidance, the guided ash-doc study path |

The primary path can assume prior programming experience. The novice path must
explain its prerequisites and offer a gradual route. Readers should be able to
move between paths without encountering contradictory explanations.

## Public structure

| Area | Proposed route | Responsibility |
| --- | --- | --- |
| Home | `/` | A concrete introduction, representative code, current maturity, and clear next steps |
| Discover | `/discover/` | Goals, language character, key ideas, and evidence-backed reasons to explore |
| Learn | `/learn/` | A concise tour, setup entry points, and routes for different experience levels |
| Documentation | `/docs/` | Installation, tool usage, practical guides, and reference/specification navigation |
| Status | `/status/` | Implemented capabilities, experiments, limitations, and development direction |
| Updates | `/updates/` | Dated releases, development reports, and design articles |
| Contribute | `/contribute/` | Entry points for language, tooling, documentation, and research contributions |
| Study Ash | `/study/` | Edition selection and access to ash-doc HTML and book downloads |

These are content areas, not a requirement for eight equal top-level menu
items. Keep primary navigation short; use the home page, contextual links, and
footer for secondary entry points.

Learn helps a visitor get started and choose a path. Study Ash provides a
sustained learning work with its own reading order. Documentation supports
task completion and lookup. The website may summarize a study topic and link
to its complete treatment without reproducing the chapter.

## Ownership and authority

Ash-site owns the evolving public account. Ash-doc owns its study publications.
Ash-lang owns normative semantics and compiler behavior. See the
[integration contract](ash-doc-integration.md).

Language claims must identify their governing specification or checkpoint.
Tool instructions must correspond to tested tool behavior. Current status
pages identify an exact inspected revision and review date. An upstream move
does not silently update a website claim or study edition.

Distinguish specified and implemented behavior, bounded experiments, accepted
future work, and exploratory ideas. A roadmap is an account of direction and
dependencies; it does not imply delivery dates or completed functionality.
Keep dated updates as historical articles, with corrections or supersession
links when needed.

## Content conventions

- Prefer original, concrete explanations and small representative programs.
- Use ash-doc's problem-driven teaching and evidence discipline as influence.
- Keep the first encounter readable; link deeper semantics and research.
- Give site-owned runnable examples canonical source, an exact compiler
  target, and checks for the behavior promised by the surrounding prose.
- Reuse an upstream example by referencing or importing its exact source;
  avoid independently maintained copies of the same program.
- Keep verification records outside the reading flow. Display target and
  maturity information where readers need it to interpret the page.
- Use meaningful headings, keyboard-accessible navigation, readable code,
  image descriptions, and layouts that work on narrow screens.

## Proposed first-version scope

Ship a complete informational core: home, an Ash overview, a short executable
tour, learning-path guidance, documentation entry points, dated language
status, an updates index with an honest launch article, contribution guidance,
and a study landing page.

Embed a study edition only after ash-doc's publication requirements are met.
Until then, the study landing page can describe the available source material
and link to a reviewed source revision, clearly marked as unpublished. The
website launch does not require publishing a book.

Defer a browser compiler/playground, user accounts, comments, a CMS, and a
complete rewrite of the language reference. These need separate user outcomes
and designs. Visual design and production hosting remain implementation
decisions rather than established facts.
