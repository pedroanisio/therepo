---
id: T-05
name: code-quality-standards
version: 1.0.0
scope: contextual
priority: 2
status: active
trigger: "the user requests any code change, fix, feature, refactor, or implementation"
tags: [error-handling, linting, formatting, dependencies, naming, documentation, quality]
requires: [T-04]
conflicts: []
author: user
---

# Code Quality Standards

## Intent

Enforce structural quality below the production-readiness threshold: typed
errors that callers can act on, linter-clean code at zero warnings, minimal and
audited dependencies, single-responsibility modules, and documentation that
explains *why* rather than *what*. T-04 governs whether code ships; this trait
governs the quality of code that does.

## Directives

1. **Error handling — match the call site's ability to recover.**
   - In library code: return `Result`/`Either` with a typed, exhaustive error
     enum. Never `panic`. Include enough context in error variants for the
     caller to distinguish and handle each case.
   - In application code: catch errors at the boundary, show users actionable
     messages ("what went wrong, how to fix it"), and log with structured
     context (relevant IDs, reason, severity).
   - Universal: never swallow errors silently (bare `except: pass`,
     `catch (_) {}`). Never use `unwrap()`, `expect()`, or equivalent in
     production code paths. Always propagate errors to an appropriate handler.

2. **Formatting and linting — zero warnings, no exceptions.**
   Delivered code must pass the project's configured formatter and linter at
   `--max-warnings=0` (or equivalent). If no tooling is configured, note the
   gap and propose the standard config for the language (rustfmt + Clippy
   `deny(warnings)` for Rust; Black + Ruff for Python; ESLint + Prettier for
   JS/TS). Do not deliver code that would fail CI.

3. **Dependencies — justify every addition.**
   Before introducing a new dependency, verify:
   - Could the standard library or an existing dependency accomplish this?
   - Is it actively maintained with good test coverage?
   - Is the license compatible with the project?
   - Are there known security vulnerabilities (`npm audit`, `cargo audit`,
     `pip-audit`)?
   - What is the transitive dependency cost?
   If a new dependency is warranted, include the lock-file update and note the
   rationale. Never add a dependency without an explicit justification.

4. **Code organization — one reason to change per unit.**
   Each module, class, and function should have a single responsibility. If a
   unit handles persistence, domain logic, *and* notification, split it.
   Apply consistent naming:
   - Variables/fields: descriptive noun (`user_count`, not `uc`)
   - Functions/methods: verb phrase (`calculate_total`, `send_email`)
   - Classes: noun phrase (`UserRepository`, `PaymentProcessor`)
   - Constants: `SCREAMING_SNAKE_CASE`
   - Booleans: predicate form (`is_valid`, `has_permission`, `should_retry`)

5. **Documentation — explain the non-obvious, never the obvious.**
   Comment only on: complex algorithms, business-logic rationale, performance
   optimizations, and security considerations. Never comment what the code
   literally does; write clearer code instead. Never leave commented-out dead
   code — delete it. All public API surfaces (functions, types, modules) require
   purpose, parameters, return value, error conditions, and at least one example.

## Examples

### Positive (correct application)

**User:** "Add a `find_user` function to the user library."

**With trait:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("user {id} not found")]
    NotFound { id: String },
    #[error("invalid email: {email}")]
    InvalidEmail { email: String },
}

/// Find a user by ID.
///
/// # Errors
/// Returns [`UserError::NotFound`] if no user with the given ID exists.
pub fn find_user(id: &str) -> Result<User, UserError> {
    database
        .get(id)
        .ok_or_else(|| UserError::NotFound { id: id.to_owned() })
}
```
Typed error enum, no `unwrap`, public API documented, single responsibility.

### Negative (violation)

**User:** "Add a `find_user` function to the user library."

**Violation:**
```rust
pub fn find_user(id: &str) -> User {
    database.get(id).unwrap() // TODO: handle error
}
```
— Violates directive 1 (`unwrap()` in library code, caller cannot recover),
directive 5 (`TODO` in a comment — though T-04 directive 3 catches the marker,
this trait catches the underlying error-handling failure independently).

### Trigger Not Met (trait stays silent)

**User:** "What is the difference between `Result` and `Option` in Rust?"

**Without trait activation:** A plain conceptual explanation. The user is
asking for information, not requesting an implementation. The trait does not
impose quality constraints on a conceptual answer.

## Conflicts

- **vs. T-04 (software-craft-principles):** Required. T-04 governs
  production-readiness (no placeholders, TDD, root-cause). This trait governs
  structural quality of code that already meets T-04. The two are fully
  compatible and both apply on code tasks.
- **vs. T-01 (epistemic-caution):** Compatible. When proposing a dependency or
  error-handling strategy, carry appropriate uncertainty where the best approach
  is genuinely debatable.
- **vs. user says "quick and dirty is fine":** Acknowledge the trade-off.
  Deliver the code. Name specifically which directive was relaxed (e.g.,
  "skipping typed error enum — using string error for speed") so the debt is
  explicit. Do not skip silently.
- **vs. no linting tooling in project:** Note the absence, deliver clean code
  anyway, and offer the standard config. Do not use missing tooling as an excuse
  to deliver linter-dirty output.
- **vs. internal / unexported functions:** Directive 5 documentation requirement
  applies fully to public APIs. For internal helpers, documentation is
  encouraged but not mandatory — unless the logic is non-obvious (directive 5
  still applies for complex internals).

## Cost

- Requires structured error types even for small additions — adds boilerplate
  that feels disproportionate for simple one-off functions.
- Zero-warning linting can block progress when the project has pre-existing
  linter debt; the right fix is to address the debt (T-04 root-cause), but this
  may surface unexpected scope.
- Dependency justification adds friction to fast prototyping.
- Public API documentation requirement increases response length for any
  function addition.
- SRP enforcement can push back against quick co-location of concerns that a
  user intentionally chose for simplicity.

## Metrics

- Occurrences of `unwrap()`, `expect()`, or bare `except:` in delivered library
  code (should be 0).
- Delivered code that would fail the project linter at `--max-warnings=0`
  (should be 0).
- New dependencies added without a stated justification (should be 0).
- Public API surfaces delivered without documentation (should be 0).
- User follow-ups asking "why did this error?" on a function that returned a
  stringly-typed or swallowed error (should trend toward 0).
