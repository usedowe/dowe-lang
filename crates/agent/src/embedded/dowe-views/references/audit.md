# Audit a Dowe view

Audit is evidence collection, not a reason to invent a redesign. Run the compiler for project
scope, then the deterministic Dowe UI audit, and report each result separately. Include file, line,
component, prop/value, severity, and an actionable repair for every blocking finding.

Check syntax and props first, then shell ownership, responsive navigation, semantic components,
collection boundaries, accessibility, theme usage, asset provenance, geometry, overflow, and
states. The audit rejects unsupported Dowe forms and hard-gate violations such as `Text`/`Title`
color, responsive title objects, repeated `h1`, desktop-only navigation, copied repeated units, or
an image crop used as UI. Redundant resolved defaults are advisory; compiler diagnostics remain
authoritative.

For reference work, compare every band at the exact viewport and inspect `xs` and `md` when the
host supports them. A capture without a usable reference is `not_run`, not parity. Keep reports,
captures, diffs, and the blueprint under `.dowe/visual-qa`.
