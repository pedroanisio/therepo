---
id: T-XX
name: your-trait-name
version: 0.1.0
scope: universal | contextual | on-demand
priority: 1
status: active | standby | deprecated
trigger: "(required if scope is contextual — natural-language condition)"
tags: []
requires: []
conflicts: []
author:
---

# Trait Name

## Intent

<!-- 1–3 sentences. Why does this trait exist? What behavioral failure does
it prevent? Write the *reason*, not the *mechanism*. -->

## Directives

<!-- Ordered list of imperative instructions. One action per directive.
Every directive must produce an observable change in the response. -->

1. [First directive]
2. [Second directive]
3. If [condition], then [action]. Otherwise, [default].

## Examples

### Positive (correct application)

**User:** "[realistic user message]"

**With trait:** "[response showing correct trait application]"

### Negative (violation)

**User:** "[realistic user message]"

**Violation:** "[response showing what going wrong looks like]"
— Violates directive [N] because [reason].

<!-- If scope is contextual, add: -->
### Trigger Not Met (trait stays silent)

**User:** "[message where the trigger condition is NOT met]"

**Without trait activation:** "[normal response, no trait influence]"

## Conflicts

<!-- Explicit rules for every known conflict. Format:
- **vs. [other trait or user instruction]:** [resolution rule] -->

- **vs. explicit user override:** [what happens]
- **vs. T-XX ([name]):** [resolution rule]

## Cost

<!-- Honest accounting. What does this trait trade away? If you write
"none" or "minimal", you are wrong — rewrite it. -->

- [Cost 1]
- [Cost 2]

## Activation

<!-- Optional. Custom phrases to toggle this trait. -->

- **Activate:** "[phrase]"
- **Deactivate:** "[phrase]"

## Metrics

<!-- Optional. Observable indicators that the trait is working. Must be
things visible in conversation transcripts, not internal states. -->

- [Metric 1]
- [Metric 2]