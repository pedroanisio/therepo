---
disclaimer: >-
  No information within this document should be taken for granted.
  Any statement or premise not backed by a real logical definition or
  verifiable reference may be invalid, erroneous, or a hallucination.
  This analysis represents one model's assessment of leverage and
  novelty — it may be wrong. Verify independently.
generated_by: "Claude Opus 4.7 (1M context) via Claude Code"
date: "2026-04-25"
title: "Step-Change Contribution: `repo mcp` — MCP Server with Cross-Agent Project Memory"
method: step-change-contributor/v1
source_document: "neutrons-repo-soul (PURPOSE.md, AGENTS.md, README.md, architecture.md), brief: MCP / offline / online / sync / central-server / multi-agent / memory"
---

# Step-Change Contribution: `repo mcp` — MCP Server with Cross-Agent Project Memory

The brief listed seven candidate themes (MCP, offline, online, sync,
central-server, multi-agent support, agent memory). The pipeline below
shows that two of them — MCP and agent memory — combine into a single
contribution that subsumes the other five as boundary conditions
rather than as separate features.

## 1. Gap Analysis

### 1.1 Structural map (re-cut through the brief)

`repo` already commits to four orthogonal stances that frame the
relevant design space:

- **Local-first / no-service** (PURPOSE §"Local-first, no services").
- **Convention over configuration** — one `.repo/` layout, one CLI.
- **Human + machine output symmetry** — every command supports
  `--json --plain`.
- **Agent-agnosticism by intention** — AGENTS.md opens with
  "any AI agent (Claude Code, Codex, Copilot, or custom)".

The current integration story for agents is uniform: an agent reads
AGENTS.md, learns the CLI surface, and shells out. `repo skills
deploy` symlinks bundled skills "for all detected agents" but says
nothing about *how* they consume capabilities at runtime — only how
they are placed on disk.

### 1.2 Strengths (relevant to the brief)

- **Local-first is load-bearing, not aesthetic.** It rules out
  central-server designs by axiom, which is good design hygiene —
  half of the brief's options (central-server, online sync) are
  pre-disqualified by an explicit principle, not handwaved away.
- **`.repo/storage/` exists** as a project-scoped storage namespace
  (currently used for plans). It is the natural home for any
  per-project state that should travel with the repo.
- **48 skills + 7 prompts + 2 schemas + 8 references** are already
  shipped as structured, machine-readable assets — exactly the kind
  of payload a protocol layer would expose.
- **JSON-first I/O contract** means the project already has the
  serialization discipline a protocol layer needs.
- **Git is already the sync mechanism, implicitly.** Anything under
  `.repo/` that is committed travels with the repo across machines
  and contributors. The "sync" arm of the brief is partially solved
  by construction; what is missing is a *policy* about what should
  travel.

### 1.3 Gap inventory (lens: brief)

| G-ID | Dimension | Description | Severity | Evidence |
|------|-----------|-------------|----------|----------|
| G-A | Cross-domain | The Model Context Protocol (MCP) — Anthropic's open JSON-RPC spec for exposing **resources, tools, and prompts** to LLM hosts (released Nov 2024, now broadly adopted across host implementations) — is not referenced anywhere in `repo`, despite `repo`'s entire reason for existing being to expose resources, tools, and prompts to LLM hosts. The project is reinventing a protocol layer in shell calls. | **critical** | No `mcp` token in AGENTS.md, README.md, architecture.md, ADRs, or `crates/repo-cli/src/`. The CLI is the only integration surface. |
| G-B | Asymmetry | `repo` ships skills, prompts, and references *as data*, but treats them as *files for agents to find on disk* rather than as *resources for agents to consume over a protocol*. The four resource families have rich CLI-side semantics (sync, install, verify) but zero protocol-side semantics. | moderate | `repo skills deploy` writes to `~/.agents/skills/`; from there, each agent re-discovers them on its own terms. No agent ever asks `repo` "what skills do you offer?" via a structured channel; it scans a directory. |
| G-C | Mechanism | `repo skills deploy --force` symlinks for "all detected agents" but the detection algorithm and the per-agent deployment layout are unspecified in any visible doc. As N agents grows, this becomes O(N) special cases. | moderate | architecture.md mentions deploy "flows" only; AGENTS.md §3.4 lists the verb but not the algorithm. |
| G-D | Boundary | No per-project, agent-accessible **memory** layer. `.repo/storage/` exists but is plan-only. Agents that *do* persist memory (e.g. Claude Code's `~/.claude/projects/<repo>/memory/`, Cursor's per-workspace state) keep it in private host directories: invisible to other agents, lost when the developer switches hosts, and never traveling with the repo. | **critical** | No `memory/` path under `.repo/`; no `repo memory` verb; no documented expectation that agent state belongs in the repo. |
| G-E | Asymmetry | Brief explicitly contrasts "offline" and "online", but the codebase has no manifest of which commands need network. `repo skills install` calls `npx skills add` (online); `repo health --check-updates` queries registries (online); everything else is offline. There is no `--offline` enforcement flag and no inventory of online dependencies. | minor | AGENTS.md does not mark commands as online/offline. |
| G-F | Temporal / sync | "Sync" is implicit (git carries `.repo/`) but undocumented as policy: which paths should be `.gitignore`d vs. tracked, which are per-developer vs. project-shared, what happens to memory written by one contributor when another pulls. | moderate | No `.gitignore` rules visible for `.repo/storage/` or a hypothetical `.repo/memory/`; no design note on multi-contributor convergence. |
| G-G | Boundary | "Central-server" appears in the brief but is structurally ruled out by PURPOSE §"Local-first, no services". This is a real boundary, not a gap to fill — the contribution should respect it. | n/a (asymmetry confirmed as intentional) | PURPOSE.md is explicit. |

### 1.4 Honesty check

- **G-G (no central server)** is intentional and correct.
  **[possibly intentional]** — and in fact the contribution must
  *honor* it, not relax it.
- **G-E (offline/online manifest)** is real but low-leverage. It
  could be a small documentation task; calling it a step-change
  contribution would be inflationary.
- **G-C (deploy detection algorithm)** is partially intentional —
  the maintainer may be deferring it pending more host integrations.
  The contribution can sidestep it rather than solve it (because
  MCP makes "deploy" partially obsolete: hosts pull, they don't
  receive).
- **G-A, G-B, G-D, G-F** are all addressable from artifacts already
  in the repo plus the published MCP specification.

---

## 2. Leverage Ranking

### 2.1 Candidate contributions

| C-ID | Form | Addresses | Cross-domain source | Novelty claim |
|------|------|-----------|---------------------|---------------|
| C1 | `repo mcp serve` — a stdio MCP server exposing `.repo/` skills as resources, `.repo/prompts/*` as MCP prompts, and `repo health` / `repo verify` / `repo docs search` as MCP tools. Plus `repo mcp install <host>` to register the server in well-known host configs. | G-A, G-B, G-C | MCP spec (modelcontextprotocol.io, Anthropic Nov 2024). | Reframes `repo` from "CLI agents shell out to" into "protocol agents speak natively". |
| C2 | `.repo/memory/` namespaced agent memory — file-based, type-tagged, exposed via `repo memory {read,write,list,prune}` CLI verbs. Sync via git. | G-D, G-F | Agent memory patterns (Claude Code's auto-memory model; emacs org-roam; Obsidian). | Names a slot the project's data model is missing. |
| C3 | **C1 + C2 unified.** `repo mcp serve` additionally exposes `.repo/memory/<agent-id>/` and a shared `.repo/memory/_common/` as MCP resources (`memory://...`) **and** as MCP tools (`memory.write`, `memory.search`). Memory becomes cross-agent (Claude writes, Cursor reads), project-scoped (lives in `.repo/`), and git-syncable (a contributor's pull updates everyone's project memory). | G-A, G-B, G-C, G-D, G-F | MCP + memory pattern; CRDT-free convergence via append-only typed records. | Treats MCP as the *delivery mechanism* for cross-agent memory, not as an alternative to it. |
| C4 | `.repo/agents.toml` + `repo agents adapt` — generates per-agent rules files (CLAUDE.md, AGENTS.md, `.cursorrules`, `.windsurfrules`) from a single source. | G-B (partial), G-C | Document templating. | Centralizes per-agent customization. Lower-impact than protocol-level unification. |
| C5 | `repo sync` over a peer git remote with explicit `.gitattributes` rules for `.repo/memory/`. | G-F | Git internals. | Mostly redundant with vanilla git; adds policy, not capability. |
| C6 | Offline/online command manifest + `--offline` enforcement flag. | G-E | None significant. | Low-impact bookkeeping. |

### 2.2 Falsifiability gate

| C-ID | Status | Refuting observation |
|------|--------|----------------------|
| C1 | **[Falsifiable]** | If any MCP-compatible host (e.g. Claude Desktop, Claude Code, or a third-party MCP client) cannot enumerate `repo`'s tools/resources/prompts without host-specific shim code, the protocol-level integration claim is refuted. |
| C2 | **[Falsifiable]** | If two `repo memory write` calls from different processes lose data (last-write-wins on the same file) the durability claim is refuted. |
| C3 | **[Falsifiable]** | If Claude (via Claude Code's MCP client) writes a memory through `repo mcp` and a second agent host (any other MCP-capable host) reading the same project cannot retrieve byte-identical content via the `memory://` resource, the cross-agent memory claim is refuted. |
| C4 | **[Falsifiable]** | If two regenerations of a per-agent rules file from the same `agents.toml` produce non-byte-equal output, refuted. |
| C5 | **[Falsifiable]** | If two contributors editing different memory files produce a git merge conflict that is unresolvable by the documented policy, refuted. |
| C6 | **[Falsifiable]** trivially. | Network call observed under `--offline` flag = refuted. |

All advance.

### 2.3 Scoring and selection

| C | Impact | Uniqueness | Realizability | Leverage |
|---|--------|------------|---------------|----------|
| C1 | 4 | 4 | 4 | 64 |
| C2 | 3 | 2 | 5 | 30 |
| **C3** | **5** | **4** | **4** | **80** |
| C4 | 3 | 2 | 5 | 30 |
| C5 | 2 | 2 | 3 | 12 |
| C6 | 2 | 1 | 5 | 10 |

**Selected: C3.**

Justification of scores:
- *Impact = 5.* Transforms `repo`'s core value proposition. The
  brief asked "support for all AI-AGENTs"; MCP **is** the answer the
  industry has converged on for that question, and adding cross-agent
  memory on top makes the contribution worth more than the sum of
  its parts (an agent-agnostic capability surface AND an
  agent-agnostic learning surface, served from one process).
- *Uniqueness = 4.* MCP-as-server is increasingly an obvious move
  (uniqueness alone ≈ 3); using MCP as the *transport for shared
  agent memory*, rather than as a separate concern, is the
  reframing that lifts uniqueness. Most projects treat memory as a
  host-private feature; treating it as a project-scoped MCP
  resource is non-obvious from inside the current frame.
- *Realizability = 4.* The spec, the on-disk format, the resource
  URI scheme, the tool schemas, and the host-config writer can all
  be produced from artifacts in the repo plus the published MCP
  spec. One implementation choice (which Rust MCP crate to depend
  on) requires the maintainer to verify against the current
  ecosystem; one citation (the live MCP spec at
  modelcontextprotocol.io) needs reader verification.

C1 alone scores 64; C3 leads by 25 % — well outside the
tiebreaker band.

---

## 3. The Contribution

### `repo mcp` — A Protocol Surface and Cross-Agent Memory for `.repo/`

#### 3.1 What is being added

Three things, deployed together:

1. A new top-level subcommand family `repo mcp` with verbs
   `serve`, `install`, `uninstall`, `list`, `inspect`.
2. A new on-disk namespace `.repo/memory/` with a typed,
   append-friendly file layout.
3. A new `repo memory` CLI verb family (`read`, `write`, `list`,
   `search`, `prune`) that operates on `.repo/memory/` directly.
   The MCP server reuses these primitives.

The split — protocol surface (`mcp`) vs. data primitive
(`memory`) — keeps the contribution composable: `repo memory`
works without an MCP host, and `repo mcp serve` works without
memory ever being used.

#### 3.2 The MCP server (`repo mcp serve`)

Transport: **stdio** (line-delimited JSON-RPC 2.0 per MCP spec).
HTTP/SSE transports are out of scope for v1 because they would
introduce a network listener — a soft violation of the
local-first commitment. A future v2 may add an opt-in
`--transport http --bind 127.0.0.1:<port>` mode for hosts that
require it; the v1 design must not foreclose this.

The server advertises three primitive sets at handshake:

##### Resources

| URI scheme | Maps to | Read semantics |
|------------|---------|----------------|
| `repo://docs/plans/<id>` | files under `.repo/storage/plans/` | Markdown body + parsed frontmatter as a JSON sidecar resource. |
| `repo://docs/adrs/<id>` | `_docs/adrs/<file>.md` | Markdown body. |
| `repo://docs/designs/<id>` | `_docs/designs/<file>.md` | Markdown body. |
| `repo://docs/references/<id>` | `_docs/references/<file>.md` | Markdown body. |
| `repo://skills/<name>` | `.repo/skills/<name>/` | The skill's `SKILL.md` (or analogous root file) plus a directory listing of bundled assets. |
| `repo://schemas/<name>` | `.repo/schemas/<name>.ts` | TypeScript schema text. |
| `repo://references/<name>` | `.repo/references/<name>.md` | Reference document text. |
| `repo://memory/<agent-id>/<type>/<id>` | `.repo/memory/<agent-id>/<type>/<id>.md` | Memory record (see §3.4). |
| `repo://memory/_common/<type>/<id>` | `.repo/memory/_common/<type>/<id>.md` | Shared cross-agent memory record. |

Resource listing (`resources/list` per MCP) returns the union of all
the above, scanned lazily from disk. Each entry includes the
standard MCP fields: `uri`, `name`, `description`, `mimeType`.

##### Tools

| Tool name | Purpose | Input schema (sketch) | Side effects |
|-----------|---------|------------------------|--------------|
| `repo.health.check` | Run `repo health --json` and return the report. | `{ "verbose": bool? }` | none |
| `repo.skills.list` | Return declared + installed skill diff. | `{}` | none |
| `repo.skills.sync` | Run `repo skills sync --json`. | `{}` | mutates `.repo/skills.toml` |
| `repo.docs.search` | Free-text + frontmatter filter over `_docs/` and `.repo/storage/`. | `{ "query": str, "kind"?: str, "status"?: str }` | none |
| `repo.verify` | Run `repo verify --json` (depends on the lockfile contribution; if absent, returns "not configured"). | `{}` | none |
| `repo.ulid` | Generate one or more ULIDs. | `{ "count"?: int }` | none |
| `repo.memory.write` | Append a typed memory record. | `{ "agent_id": str, "type": "user"\|"feedback"\|"project"\|"reference", "name": str, "description": str, "body": str, "scope"?: "private"\|"common" }` | writes file under `.repo/memory/` |
| `repo.memory.read` | Read a single memory record by URI. | `{ "uri": str }` | none |
| `repo.memory.search` | Grep memory by type, agent, or substring. | `{ "agent_id"?: str, "type"?: str, "query"?: str }` | none |
| `repo.memory.prune` | Mark a memory record as superseded. | `{ "uri": str, "reason": str }` | adds tombstone (see §3.4) |

Tools that would mutate state outside `.repo/` are deliberately
absent. `repo.skills.install` is **not** exposed as a tool because
it shells out to `npx` — a network operation that an LLM should
not initiate without an explicit human gesture. The CLI verb
remains; the protocol surface omits it. This is the v1 boundary
between "things an agent may do" and "things a human runs."

##### Prompts

Every file under `.repo/prompts/` is exposed as an MCP prompt.
The prompt name is the filename stem; arguments are derived from
`{{var}}` placeholders in the body (or, if a `prompt.toml` sidecar
exists, from a declared schema). MCP `prompts/get` returns the
rendered string with arguments substituted. The seven built-in
prompts (`assess-corpus`, `audit-repo`, `feedback-processor`,
`format-plan`, `review-cycle`, `review-internal`, `validate-plan`)
become first-class agent-callable templates with zero additional
work.

#### 3.3 Host integration (`repo mcp install <host>`)

The `install` verb writes the server registration into the host's
canonical MCP config. Targets in v1:

| Host | Config target | Notes |
|------|---------------|-------|
| `claude-desktop` | `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS), `%APPDATA%\Claude\claude_desktop_config.json` (Windows) — the `mcpServers` object. | Adds `{ "repo": { "command": "repo", "args": ["mcp", "serve"] } }`. |
| `claude-code` | Project `.mcp.json` or user `~/.claude.json` `mcpServers` field, depending on `--scope`. The `claude mcp add` CLI also exists; `repo mcp install claude-code` should produce the same JSON either way. | Default scope: `project`. |
| `cursor` | Per-project `.cursor/mcp.json` (or the user-level equivalent). | The exact path/schema should be verified against current Cursor MCP docs before shipping; the CLI must read its target from a small per-host adapter table that is straightforward to update. |
| `windsurf`, `zed`, `continue` | Same pattern: small adapter table mapping host id → config path + JSON shape. | One file to edit when a host's config moves. |

`repo mcp install --all` installs into every host whose config
file exists on the current machine. `repo mcp uninstall <host>`
removes the entry. `repo mcp list` prints, in JSON or table form,
which hosts are currently registered.

This is the **only** per-host special-casing in the design: a
small adapter table for *configuration file paths*. Once the
server is registered, every host speaks the same wire protocol —
there is no per-host dispatch, no per-host skill layout, no
per-host prompt format. G-C dissolves: `repo skills deploy`
remains for hosts that want files on disk, but new hosts just
need to consume MCP and they get everything for free.

#### 3.4 The memory layer (`.repo/memory/`)

##### Layout

```
.repo/memory/
  MEMORY.md                     # human-readable index (one line per record)
  _common/                      # cross-agent memories
    user/
      user-role.md
    feedback/
      no-mocks-in-integration-tests.md
    project/
      auth-rewrite-driver.md
    reference/
      grafana-latency-board.md
  claude-code/                  # agent-id-namespaced memories
    user/
    feedback/
    project/
    reference/
  cursor/
  copilot/
  _tombstones/                  # pruned records (see below)
```

The four type buckets — `user`, `feedback`, `project`,
`reference` — match the taxonomy already documented in Claude
Code's auto-memory system. Borrowing a working taxonomy avoids
reinventing one and keeps the format intelligible to the host
that will most often produce records. **[Assumption introduced by
contributor]:** other agent hosts will accept a four-type memory
schema. If they object, the schema is forward-compatible by
accretion (additional types can be added without breaking
existing readers).

##### Record format

Every memory record is a single Markdown file with frontmatter:

```markdown
---
name: integration-tests-no-mocks
description: integration tests must hit a real DB, not mocks
type: feedback
agent_id: claude-code
scope: common
created_at: 2026-04-25T14:32:11Z
created_by: claude-opus-4-7
supersedes: []
---

Integration tests must hit a real database, not mocks.

**Why:** prior incident where mock/prod divergence masked a
broken migration in Q4 2025.

**How to apply:** any test under `crates/*/tests/` that touches
persistence MUST use the testcontainer fixture, never `mockall`.
```

Required frontmatter fields: `name`, `description`, `type`,
`agent_id`, `scope` (`private` | `common`), `created_at`. The
body is free-form Markdown.

##### Convergence (the "sync" question)

Memory records are append-only at the file level. Each record
lives in its own file, ULID- or slug-named within its
`<agent>/<type>/` directory. Two contributors creating different
memories never collide; two contributors creating the *same*
memory produce a normal git conflict, resolved by keeping the one
with the earlier `created_at` and superseding the later one.

To **revise** a memory, write a new record whose `supersedes`
array contains the URI of the old record, then move the old
record to `_tombstones/`. `repo memory prune` automates this. Any
reader is expected to filter out records superseded by something
in their own history.

This is a deliberately primitive convergence story — append-only
files, git for transport, supersede-by-pointer for revision. It
is the simplest mechanism that satisfies the brief without
introducing CRDT machinery, a server, or a custom sync protocol.
Cite: the same pattern is used by message-passing systems (e.g.
the Matrix protocol's event-replacement model) and by some
note-taking systems (Obsidian's "version" plugins). The reader
should verify the specific Matrix replacement spec at
spec.matrix.org if they want to mirror its exact field names.

##### What goes in `.gitignore`

The contribution explicitly addresses G-F:

- `.repo/memory/_common/` and `.repo/memory/<agent>/` — **tracked
  by git by default.** This is the project's institutional memory.
- `.repo/memory/_tombstones/` — **tracked** (so contributors agree
  on what was retracted).
- `.repo/storage/cache/`, `.repo/storage/sessions/` — **ignored**
  (per-developer ephemera).

A `.repo/.gitignore` template ships with `repo memory init` to
make the policy mechanical, not aspirational.

#### 3.5 What this does NOT change

- The CLI surface stays. `repo skills`, `repo health`, `repo
  prompt`, `repo docs` keep their current verbs and outputs. The
  MCP server is a *new face* on the same logic, not a replacement.
- The local-first principle stays. Stdio transport means no
  process listens on a network port in v1. `repo skills install`
  remains the only verb that touches the network, and it is not
  exposed as an MCP tool.
- The "no central server" principle stays. Sync is git, end of
  story. Any future "remote memory" feature would be a separate
  decision and explicitly opt-in.

#### 3.6 Falsification criterion

**[Falsification criterion]:** Configure two MCP-capable hosts
on the same machine pointed at the same `.repo/`-managed project
(e.g., Claude Desktop and Claude Code, or Claude Code and any
third-party MCP client). From host A, call the MCP tool
`repo.memory.write` with a known `name`, `type`, and `body`. From
host B, call `repo.memory.read` for the resulting `repo://memory/...`
URI. If the returned content is not byte-identical to what host A
wrote (modulo the `created_at` timestamp), or if host B cannot
discover the URI via `resources/list`, the cross-agent memory
claim is refuted.

A second, independent criterion: enumerate all tools, resources,
and prompts via MCP from at least two hosts. If the enumerations
disagree on tool names, schemas, or prompt argument lists, the
"single protocol surface" claim is refuted (the server would be
host-aware, defeating the purpose).

---

## 4. Justification

### 4.1 Step-change argument

The current `repo` is an excellent CLI. Every agent it supports
must (a) know to call it, (b) parse its `--json`, (c) translate
its outputs into the agent's own internal model of "what
capabilities does this project offer me?" Each new host pays the
same integration tax. Each host loses the cognitive work the
others have done — a memory written by Claude during an
investigation is invisible to Cursor in the next session.

After the contribution, `repo` is *also* a protocol server that
speaks the standard every modern LLM host already speaks. The
integration tax collapses to "register the server in the host's
config" — one line of JSON, automated by `repo mcp install`.
Capability discovery happens through MCP's standard
`resources/list` / `tools/list` / `prompts/list`. The host does
not need to learn `repo`; it needs to support MCP, which it
already does.

The memory layer is what makes the move worth more than its
parts. Without it, MCP is just a thinner adapter for the same
CLI. With it, the protocol carries something CLIs cannot easily
carry: durable, typed, cross-agent state that lives in the repo,
travels with `git pull`, and is queryable through the same
channel as everything else. An agent learning something useful in
session 1 — "this codebase forbids mocks in integration tests" —
becomes a fact every other agent on every other contributor's
machine sees in session 2. Project knowledge stops being
host-private.

The improvement is non-linear because it dissolves three problems
the project would otherwise have to solve sequentially: (i)
per-host integration shims (G-A, G-C), (ii) durable agent state
(G-D), and (iii) sync policy (G-F). MCP gives (i) for free; the
memory layer gives (ii); putting memory under `.repo/` and
designating it tracked-by-git gives (iii). One contribution, three
gaps closed, no new principles violated.

### 4.2 Leverage audit

- *Impact = 5* held up. The contribution does what the brief asked
  ("support for all AI-AGENTs", "memory for AI-AGENTs") in one
  move that respects every existing principle.
- *Uniqueness = 4* held up. MCP-as-protocol-surface alone is
  increasingly unsurprising; MCP-as-carrier-for-cross-agent-memory
  remains a non-obvious composition. The `repo://memory/` URI
  scheme is the load-bearing piece — it turns memory from a host
  feature into a project resource.
- *Realizability = 4* held up with one honest mark-down: per-host
  config paths drift over time (Cursor's MCP config layout in
  particular has moved more than once during MCP's first 18
  months). The adapter table for `repo mcp install` will need
  maintenance. The protocol surface itself is stable; only the
  install-side glue is volatile.

### 4.3 Falsifiability audit

- **Falsification criterion (restated):** Cross-host write/read
  round-trip via MCP must produce byte-identical content; cross-host
  enumeration of tools/resources/prompts must agree.
- **Test sketch:** Add an integration test
  `crates/repo-cli/tests/mcp_cross_host.rs` that spawns `repo mcp
  serve` as a subprocess, drives it from two independent
  in-process MCP clients (the spec is small enough that a hand-
  rolled test client is feasible; alternatively, depend on a
  community Rust MCP client crate), and asserts:
  (1) `tools/list` returns identical schemas across both clients;
  (2) `resources/list` returns identical URIs;
  (3) `tools/call` `repo.memory.write` from client A followed by
  `resources/read` from client B returns matching bytes; and
  (4) the underlying file under `.repo/memory/` is created with
  the documented frontmatter shape. Hermetic, deterministic, no
  network. Runnable under the existing 91 % coverage policy.
- **Strength:** **Strong.** A single counter-observation refutes
  the core claim. There is no hedging room; either both clients
  see the same surface and the same content, or they do not.

### 4.4 Limitations and caveats

- **Does not solve external plugin execution.** That gap (G5 in
  the prior contribution) remains intentionally deferred. MCP is
  *not* a plugin protocol for `repo` itself; it is a protocol for
  exposing `repo`'s data to LLM hosts.
- **Does not introduce remote memory.** Multi-machine sync is
  git, period. If two contributors want shared memory without
  sharing a repo, they need an out-of-band mechanism this
  contribution explicitly declines to provide.
- **Adapter table is volatile.** Per-host MCP config paths and
  schemas evolve. The adapter table needs an active maintenance
  posture; a contributor likely needs to update it once per quarter
  in the protocol's current adoption phase.
- **Network-touching tools intentionally excluded from MCP.**
  `repo skills install` and `repo health --check-updates` remain
  CLI-only in v1 because letting an LLM trigger arbitrary
  network operations through a project tool is a meaningful
  expansion of the trust surface. A future v2 may expose them
  behind an explicit `--allow-network` server flag, but that is a
  separate decision with separate threat-modeling.
- **Memory schema is opinionated.** The four-type taxonomy
  (`user`, `feedback`, `project`, `reference`) is borrowed from
  Claude Code's auto-memory system. Other agents may want
  different types or different granularity. The format is
  forward-compatible by accretion, but the choice is non-neutral
  and may need revisiting if the dominant memory-producing host
  changes.
- **Convergence is primitive.** Append-only files + supersede-by-
  pointer is the right v1 mechanism but will not scale to
  high-frequency memory writes from many concurrent hosts. If
  agents start producing dozens of memories per session per host,
  a richer convergence model (CRDTs, log-structured merge) may be
  needed. v1 is sized for human-rate writes (a few per hour per
  agent), which matches the current usage pattern of every memory
  feature shipped to date.
- **Blind spots of the contribution itself.** This analysis
  prioritizes protocol-level uniformity and project-portable
  memory because both have well-developed cross-domain analogues.
  A maintainer who values per-host customization (different
  agents seeing different views of the project) would legitimately
  weight differently. The contribution is correct as a closure of
  the brief's stated themes, not as a final word on multi-agent
  collaboration design.
