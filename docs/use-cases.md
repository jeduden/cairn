---
summary: >-
  Pending use cases, intents and flows, one file each under
  docs/use-cases, before the SRS covers them: who wants each, what it
  is, the milestone it targets and its status. Not normative; an entry
  moves into the SRS when work picks it up.
---
# Use cases

This index collects intents the SRS does not cover yet, so that none of
them lives only in a chat. It is not normative: the SRS decides what
Cairn does. When work picks an entry up, it lands as a domain-model
concept, requirement rows and `@pending` scenarios, and its entry then
names those requirement ids.

## Format

Each use case is one file, `docs/use-cases/UC-<nnn>.md`. It holds only
its front matter and its title. mdsmith checks the front matter against
the `use-case` kind in `.mdsmith.yml`. The table below is built from
it. The fields:

- **id:** `UC-` and three digits, matching the file name; never reused.
- **title:** a short name, repeated as the file's heading.
- **summary:** the use case, one sentence of at most 40 words, with no
  design in it.
- **personas:** one or more of U1 to U9, as SRS §2.5 lists them.
- **milestone:** one milestone of SRS §12.2 (`M7`), a range
  (`M7 to M9`), or `unplanned` while none is chosen yet.
- **status:** `pending`, `specified` or `shipped`.
- **requirements:** left out while pending; once the entry is
  specified, the requirement ids that carry it.

## Entries

<?catalog
glob: "use-cases/UC-*.md"
sort: id
header: |
  | Id | Use case | Personas | Milestone | Status | Requirements |
  |----|----------|----------|-----------|--------|--------------|
row: "| [{id}]({filename}) | {summary} | {personas} | \
  {milestone} | {status} | {requirements} |"
?>
| Id                            | Use case                                                                                                                                                 | Personas   | Milestone | Status  | Requirements |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | --------- | ------- | ------------ |
| [UC-001](use-cases/UC-001.md) | Keep the images an agent saw or a person pasted in the record, handled safely although no pattern can redact them, and shown or recalled only on request | U5, U8, U9 | M1        | pending |              |
| [UC-002](use-cases/UC-002.md) | Mark a span of text inside an event, or a region of an image, and comment on exactly that                                                                | U5, U6     | M7        | pending |              |
| [UC-003](use-cases/UC-003.md) | Share a link to a marked section and its comment that carries only ids and opens only for people with access                                             | U2, U5, U6 | M7 to M9  | pending |              |
| [UC-004](use-cases/UC-004.md) | Run agent work on a schedule, such as a nightly task, under a rule a person recorded in advance, with no person present                                  | U1, U2     | unplanned | pending |              |
| [UC-005](use-cases/UC-005.md) | Start agent work or post a notice in a room when a forge event arrives, such as a new issue or a merged pull request                                     | U2, U5, U7 | unplanned | pending |              |
<?/catalog?>
