---
title: "Pins and context"
order: "05"
summary: >-
  What may reach a model: pins and their versions, types and budget, the intent, stakes, posts, room summaries, envelopes, restore blocks, opt-in notices and held requests.
---
# Pins and context

- **Pin**: Verbatim text in exactly one room, with a pin type and numbered pin
  versions, each with one author (a seat); the pin's author is its first
  version's, and only it, or for a device-seat pin its author's principal from
  any of its devices whose device scope allows it, edits the pin, except the
  intent (Intent). Information, never an instruction. It restores only under
  PIN-10, to runs that have had a seat in its room during the run, as a
  **qualifying pin**, its creating event or stamp trusted on this node: a pin of
  a type that restores written from a device seat a device key certified, within
  that key's device scope, restores to its author's principal's agents, the
  intent as Intent says (before PRV-10 ships, a pin from this node's own device
  seat to this node's agents), and to agents whose principal's trust grant
  covers its author's principal; any version of a type that restores restores to
  the agents of a principal who stamped it. Adding, editing or unpinning a pin
  of a type that restores, written from a device seat a device key certified
  (before PRV-10 ships, this node's own device seat), is a widening principal
  act of its author's principal, and a verdict and a pin candidate's
  confirmation are their own principal acts (OWN-27, PIN-05); each, but a
  neutral unpin, still needs its seat's pin capability; every other pin, a
  token-key-only node's included, is changed by room acts, but for its
  principal's neutral unpin of its own agent's run-seat pin, and a run seat's or
  a token-key-only node's restores only once stamped.
- **Pin version**: One immutable text of a pin; each edit adds one, unstamped.
  What a stamp covers.
- **Pin candidate**: Proposed pin text an agent suggested or Cairn detected; not
  yet a pin, and with no pin author. The confirmation of its principal (of the
  agent that suggested it or the node that detected it), a widening act, makes
  it a new pin its device seat authors; for an intent or a criterion, only the
  owner's confirmation, making a new version of the room's intent pin that the
  owner's device seat authors.
- **Configuration pin**: A pin the principal's configuration declares, authored
  by its device seat on that node (PIN-01).
- **Pin priority**: An integer the pin's author sets with it, lower first, the
  intent before every other pin (LANE-20); qualifying pins fill the pin budget
  in that order, then by their creating events' addresses (PIN-08).
- **Budget**: The **pin budget** is the restore block's share of model tokens;
  every other budget is named too: the **hook budget** (a hook handler's time
  limit, §9.1, NFR-02), the **step budget** (a kernel execution's step limit,
  CMP-05) or the delegation budget; a limit keeps its own name, such as the
  **restore block limit** (INJ-07), CMP-05's wall-clock and memory limits,
  SEC-04's query deadline and the output caps (RCL-03, CMP-06). I3's budget is
  the pin budget, and I9's defined budgets are these named budgets and limits;
  the latency targets of NFR-01, NFR-03 and NFR-15 are targets, not budgets.
- **Pin type**: One of `constraint`, `preference`, `decision`, `fact`,
  `episode`, `intent`, `verdict` and `stake`. Only `constraint`, `preference`
  and `intent` pins restore.
- **Unpin**: The act of a pin's author, or its author's principal, that ends the
  pin, the intent excepted (Intent): it takes the pin off its room's **pin
  list** (its pins neither unpinned nor taken off by a list removal, the intent
  first) and stops it restoring, while its versions stay in the record (I1). A
  stamped version keeps restoring to its stamper's agents until the stamper
  unstamps it, raising a Needs you item.
- **List removal**: A moderator's or the owner's room act taking another seat's
  pin off the pin list without unpinning it, never the intent. A device-seat pin
  it removes keeps restoring to every agent it restored to until its author's
  principal unpins it, raising a Needs you item.
- **Intent**: A room's lead pin, of type `intent`: a goal, its criteria and
  optionally the paths it is meant to change (LANE-20). Only the owner's
  principal act changes it; a new owner's revision adds a version its device
  seat authors. The intent restores as the author of its newest version wrote
  it, to that author's principal's agents; after a handover, until the new owner
  revises or stamps it, it restores only to its stampers' agents.
- **Criterion**: One acceptance condition of an intent, with a stable id.
- **Stake**: A pin of type `stake` stating what its author works on. Only its
  author, or for a device seat its author's principal, edits it; it is unpinned
  as Unpin says, and it makes Cairn refuse no other seat's act.
- **Title, labels**: Room state the owner's device seat or a moderator sets: a
  display name and tags. They never reach a model.
- **Assignment**: Room state asking a seat to work on a branch. It makes Cairn
  refuse no seat's act; a role assignment is always named so.
- **Post**: Text a seat writes to a room's conversation, with provenance `post`,
  reaching an agent only by recall, endorsement or a trust grant (OWN-29). A
  post may carry range links; a **comment** is a post on a **marked range**, the
  address range a range link names. A seat's unsent post is a **draft**,
  ephemeral like a presence hint (PEER-09). A **cross-room post** stays in the
  sending seat's writer; the target room shows it, and its seats pull it by
  address, enveloped (LANE-29).
- **Directed post**: A post directed to one agent. It waits in that agent's
  principal's Needs you queue for an endorsement (LANE-12).
- **Room summary**: A facilitator's summary of a room, always untrusted, linked
  by address to the events it covers, read only through `room_summary_get`,
  which never writes; a summary request is `room_summary_request` (LANE-33). A
  restore block's **room summary pointer** names the latest room summary by id
  and version.
- **Compaction summary**: The harness's summary at compaction, recorded as
  untrusted `harness_text`, never restored.
- **Envelope**: The **untrusted-data envelope** of I2, the one name beside
  "envelope": the wrapper that marks recalled content as untrusted historical
  data; the only way recalled content reaches a model. An envelope **marked
  structural** carries only structural fields, §9.7's closed status words and
  marks, and Cairn's ids (`room_get`, VIEW-15).
- **Envelope warning**: The fixed sentence at the head of every envelope (field
  `warning`).
- **Trusted text** (`TrustedText`): text the core's **restore builder** makes
  (INJ-03) only from qualifying pins, sanitized structural fields and fixed text
  Cairn ships: restore blocks, opt-in notices, compaction guidance and the fixed
  templates of OWN-04 and OWN-07, never a post's text.
- **Restore block**: Deterministic trusted text, built only from qualifying
  pins, sanitized structural fields and fixed text Cairn ships, that Cairn
  injects after compaction, at a run's start or harness resume (INJ-02), and on
  a prompt while `restore_block.on_prompt` is on (INJ-04), carrying among other
  things qualifying pins, their room ids and the run's seat ids, omitted pins'
  ids and count, the count, room id and key fingerprint of pins of a type that
  restores that do not qualify (PIN-10), PIN-11's count and reason, PEER-06's
  note that later pins may be missing, a landmark index, a recall hint and
  LANE-33's room summary pointer.
- **Landmark index**: The current run's landmarks, each with its address range,
  as a restore block lists them (INJ-01).
- **Recall hint**: The one fixed line in a restore block saying that the recall
  tools reach the full history (INJ-01).
- **Opt-in notice**: Trusted text built only from fixed text Cairn ships and
  Cairn's own ids, version numbers, counts, key fingerprints and addresses,
  saying posts or a delegate report wait, or a room changed. Sent only while the
  owner's notice allowance and the agent's principal's notice opt-in both stand
  (INJ-10, LANE-30). The only meaning of "notice".
- **Model**: The language model behind an agent, such as Claude; "the model"
  means it wherever the domain model's files, or the domain-model agent's
  instructions, are not speaking of the domain model. Its output in a turn is a
  **model reply**; a **model token** is its unit of text for budgets.
- **Working view**: Whatever is currently in the model's context window. Never
  part of the record, never authoritative.
- **Held request**: A permission request, question or hand-off with a stable id,
  answerable from any principal surface within its scope (OWN-05). The agent is
  kept waiting no longer than its **hold window**, then its away policy, if one
  is on, answers the agent; the held request stays open, and with no away policy
  on the agent keeps waiting (OWN-06). A **reply** answers a held question with
  principal-typed text.
- **Qualified requests**: A permission request (the harness's, held as a held
  request), a role request (a viewer's room act asking for a wider role), a join
  request (a room act of a run's personal-room seat naming the room; it needs
  its principal's acceptance unless that principal asked for the join), an
  erasure request (a node sends its purge to its peers, PEER-11), a purge
  request (a neutral principal act asking the room's owner to purge what its
  seats wrote, SEC-30), a quarantine request (to peers), and a summary request
  (to the facilitator). "Request" never stands alone.
- **Notification**: A signal to a person: in-page, desktop or terminal on the
  same device, or through the notification bridge to a service the node's
  principal names. It carries only the room's petname, else its id, the queue
  class and a count, never room text; it never reaches a model and never accepts
  answers.
