---
name: review-plan
description: "Inspect, approve, or improve a PlanSchema JSON plan. Use whenever the user asks to review a plan, validate a plan, check a plan for issues, approve a plan, or improve/fix a plan. Also use when the user says 'review this plan', 'is this plan ready', 'check my plan', or references plan quality."
license: CC0-1.0
metadata:
  version: "1.0.0"
  domain: planning
  triggers: review plan, inspect plan, approve plan, improve plan, validate plan, check plan, plan review, plan quality, plan ready
  role: plan-reviewer
  scope: plan-validation
  output-format: markdown
ULID: 01KM1BVKWT984AB0A4WPZRWWGX
---

# Review Plan

## Overview

Inspect a PlanSchema v0.3.0 JSON plan, produce a structured verdict, and — when the verdict is IMPROVE — apply fixes and re-validate. The goal is to catch structural errors, semantic gaps, and quality issues before execution begins.

**Announce at start:** "I'm using the review-plan skill to inspect this plan."

## Inputs

The skill accepts one of:

1. **A file path** to a PlanSchema JSON (e.g., `.repo/storage/.../plan-foo.json`)
2. **An open file** in the IDE that is a PlanSchema JSON
3. **An inline JSON blob** pasted by the user

If the user says "review the plan" without specifying which, look for the most recently modified `plan-*.json` under `.repo/storage/`.

## The Review Process

### Phase 1 — Load and Parse

1. Read the plan JSON file.
2. Read the plan schema from `.iande/schemas/plan-schema.ts` — this is the authoritative reference for field semantics, branded identifiers, and the 13 well-formedness checks.
3. Confirm `schemaVersion` matches `"0.3.0"`. If not, note the mismatch and proceed with best-effort review.

### Phase 2 — Structural Validation

Run every check from `validateWellFormedness()` mentally, categorizing results:

**Hard errors (blocking):**
1. Referential integrity — all actor/step/zone/resource IDs resolve
2. Scope containment [Ax 2.2] — steps stay within actor's authorized zones
3. DAG acyclicity [Def 3.1] — no dependency cycles
4. Execution order completeness — all steps in `executionOrder.sequence`, no orphans
5. Capacity feasibility [Def 3.3, A1] — simultaneous resource tokens fit actor capacity
6. Irreversibility gating [Ax 2.3] — operator approval, no self-authorization
7. Intent projection adequacy [Def 3.3, C8] — `verificationEconomics` present with constraints
8. Snapshot consistency — `metadata.snapshotRef == baseline.snapshotRef`
9. Plan version authorization [Def 3.2] — no self-authorization in `versionHistory`
10. Bandwidth allocation [Def 4.14] — `bwDecl + bwReview ≤ bwVerify`

**Warnings (non-blocking but must be acknowledged):**
11. Detection adequacy [Prop 2.6] — self-only verification
12. Constraint debt [Def 4.2] — `valDone < valReq` accumulation
13. Handoff compression loss [Prop 2.2] — cumulative loss > 50%
14. Verification gap [Cor 4.1.1] — emit >> verify on critical channels
15. Blast radius [Def 2.8] — runtime-only, no static lower bound
16. Thrashing risk [Def 2.19] — >80% capacity utilization

### Phase 3 — Semantic Review

Go beyond structural validation. Evaluate:

1. **Problem clarity** — Is `problemStatement` specific and falsifiable? Does `successOutcome` map to `acceptanceCriteria`?
2. **Baseline accuracy** — Does `snapshotRef` match the current HEAD (or is it stale)? Are `healthCommands` executable?
3. **Scope precision** — Are `inScope` globs tight enough? Is `nonScope` explicitly justified? Are `sharedSurfaces` complete?
4. **Step quality:**
   - Size ↔ fileChanges count consistency (an XS step with 10 file changes is suspicious)
   - M+ steps must have `stopConditions` (at least 1)
   - M+ steps must have `resourceRequirements.simultaneousResources`
   - L+ steps must have at least one `verification` with `verifiedBy: "human"`
   - Session-bounded agents must have `handoffTemplate` at phase boundaries
   - `commitTemplate` follows conventional commit format
5. **Decision authority** — Vendor/cost/infrastructure/irreversible decisions must have `decidedBy` set to a human actor, not an agent
6. **Risk coverage** — Every L+ step should have at least one associated risk entry. Critical risks must have mitigation + fallback
7. **Verification economics** — Are default values used without justification? Do `intentProjection` constraints cover the success criteria?
8. **Domain entity confidence** — Do resource paths actually exist in the codebase? If the plan references files, verify they exist (use `Glob` or `Grep`)
9. **Acceptance criteria testability** — Every criterion should have a `verificationCommand` or be clearly human-verifiable

### Phase 4 — Produce Verdict

Based on findings, assign one of three verdicts:

#### APPROVE

All structural checks pass, no semantic issues found. The plan is ready for execution.

Output format:
```
## Verdict: APPROVE

**Plan:** {metadata.planId} v{metadata.version}
**Schema:** {schemaVersion}
**Snapshot:** {metadata.snapshotRef}

### Structural Validation
All 13 well-formedness checks passed.

### Semantic Review
No issues found.

### Notes
{Any observations that don't block approval but are worth noting.}
```

#### IMPROVE

Issues found but they are fixable without redesigning the plan. List every issue with a concrete fix.

Output format:
```
## Verdict: IMPROVE

**Plan:** {metadata.planId} v{metadata.version}
**Issues found:** {N errors, M warnings}

### Errors (must fix)
1. {Error description} → **Fix:** {Specific fix}
2. ...

### Warnings (should fix)
1. {Warning description} → **Fix:** {Specific fix}
2. ...

### Semantic Issues
1. {Issue description} → **Fix:** {Specific fix}
2. ...

Shall I apply these fixes now?
```

#### REJECT

Fundamental design problems that require rethinking scope, approach, or architecture. Fixing individual fields won't resolve the issue.

Output format:
```
## Verdict: REJECT

**Plan:** {metadata.planId} v{metadata.version}
**Reason:** {One-sentence summary of the fundamental problem}

### Analysis
{Detailed explanation of why the plan needs redesign, not patching.}

### Recommendation
{What should happen next — rebuild mental model, re-scope, etc.}
```

### Phase 5 — Apply Fixes (IMPROVE verdict only)

When the verdict is IMPROVE and the user confirms:

1. Read the plan file.
2. Apply each fix from the IMPROVE verdict.
3. Re-run the full structural validation mentally.
4. If new issues surface from the fixes, report and fix those too.
5. Write the updated plan back to the same path.
6. Update `metadata.updatedAt` to current timestamp.
7. Present a summary of changes made.

Do NOT increment `metadata.version` — version transitions require human authorization [Def 3.2].

## Codebase Verification

When checking domain entity confidence (Phase 3, item 8), actively verify:

- Every `resources[].path` exists in the working tree
- Every `steps[].fileChanges[].path` for `action: "modify"` references an existing file
- Every `steps[].fileChanges[].path` for `action: "create"` does NOT already exist
- Every `healthCommands` entry is a valid command

Use `Glob` for path verification. Do not guess — verify or flag as unverified.

## What This Skill Does NOT Do

- **Generate plans from scratch** — use the plan generation protocol for that
- **Execute plans** — use the `executing-plans` skill for that
- **Convert plans to markdown** — use the `format-plan` prompt for that
- **Modify plan scope or architecture** — that requires human authority

## Edge Cases

- **Stale snapshot:** If `snapshotRef` doesn't match current HEAD, warn but continue review. The plan may still be structurally sound.
- **Missing verificationEconomics:** Flag as a hard error per [Def 3.3/C8]. Suggest adding defaults from plan-generation.md §2.2.
- **Empty intentProjection:** Flag as a hard error. Every plan must have at least one grounded constraint.
- **v0.2.0 validationBudget fields:** Accept both v0.2.0 (`required`/`performed`) and v0.3.0 (`valReq`/`valDone`) notation.
