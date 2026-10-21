# Site voice

The voice of the Ash website, for its content pages: home, discover, learn
entry points, status, updates, diaries, contribute, and interface copy. The
specification, docs publications, and study book keep their own voices and
their own guides; this document does not apply to them.

The register in one sentence: the site states what the language is and does;
it does not sell, beckon, or reassure.

## Principles

1. **State, don't sell.** Headlines name things in plain nouns
   ("Static types. Effect rows. Scoped resources."). No promotional
   adjectives, no campaigns. Underclaim conspicuously where the truth is
   modest: "Working implementation. Ongoing research."
   Not: "Step inside the work between milestones", "every contribution helps
   shape Ash".

2. **Short beats up top, proof underneath.** Headlines compress; the sentence
   after them proves the claim with one concrete fact or mechanism. Never two
   abstract sentences in a row. The home page pairs its claim ("pairs
   expressiveness with assurance") with one proof ("The types are the
   contract; the code must keep it").

3. **Strong verbs; no modal mush.** "must keep it", not "has to fulfil";
   "pairs", not "combines". If a verb carries no weight, cut it or replace it.
   "Stability and production readiness are not promised" — not "may be
   unstable" or "should be expected".

4. **Jargon by depth.** Use the precise term where the reader knows it
   (effect rows, nominal, parametric), the plain phrase at the top of the
   funnel. One level of abstraction per surface. Never two names for one
   destination on a single surface ("specification" in the title is
   "specification" in the body, not "language reference").

5. **The contract vocabulary.** One metaphor family recurs across the site:
   declare, declared, contract, keep, check, honest. Echoes across pages are
   deliberate. Accidental echoes are not: two emphasised texts (hero headline,
   section heading) never share a key word.

6. **Honesty stated in the positive.** Say what exists and what does not.
   Prefer "not promised" to hedged qualifiers; prefer "evolving" to
   "cutting-edge".

7. **Precision over familiarity.** Choose the accurate word over the habitual
   one: "semantics", not "elaboration and evaluation"; "Diaries", not "blog".
   A word earns its place only when the site's own documents already use it
   ("parametric effects" appears in the specification).

8. **Rhythm is information.** Beats, line breaks, and repetition are set
   deliberately, never left to chance wraps. Repetition of a word is
   permitted when the register differs (an eyebrow label above a heading that
   contains the same word); it is cut when both texts are emphasised.

## Mechanics

- Oxford comma: "Data, requests, and resources".
- Sentence case for headings, labels, and buttons.
- en-GB date format (18 October 2026) as the pages already use.
- No exclamation marks. No emoji.
- Link labels name the action or destination in two to four words
  ("Read the tour", "All updates").

## Before shipping copy

- Read it aloud: every verb earns its place; nothing campaigns.
- Check the surface's abstraction level: exactly one.
- Check for shared key words between neighbouring emphasised texts.
- Check that every claim appears in, or follows from, the specification or
  the status page.
- Check house mechanics (comma, case, dates).

## Worked examples (October 2026 home page pass)

| Before | After | Why |
| --- | --- | --- |
| A functional language with explicit effects. | Types declare the shape. / Effect rows say what is asked. / Resources stay scoped. | Names all three pillars; beat rhythm |
| Ash combines static types and pattern matching with lexical effect handlers and scoped resources. | An evolving functional language that pairs expressiveness with assurance. | Claim at the right level of abstraction |
| A function's type tells you what it returns and what it can request. | The types are the contract; the code must keep it. | Strong verbs; contract vocabulary |
| Look up syntax, type checking, elaboration, and evaluation in the language reference. | Look up syntax, type checking, and semantics in the language reference. | Jargon by depth; one name for the destination |
| Data and patterns (term, pattern matching listed first) | Data types (generics, interfaces, inference, checking) | Checked coverage outranks sugar |
