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

## Variant and scheme checks

For every component that supports both props, exercise the same matrix: no local props, only
`variant`, only `scheme`, and both props. Confirm that omitted fields inherit independently from
the matching `design` slot or built-in default; a local `variant` must not erase a theme `scheme`,
and a local `scheme` must not erase a theme `variant`.

Include structural `background` and `surface` cases for transparent `line` and `ghost` variants,
the `Card` `outlined` structural-fill exception, and stateful/navigation components such as
Accordion, Tabs, Sidebar, NavMenu, and SideNav when they are in scope. Validate normalized IR roles
and every requested target generator, not only the source declaration or one platform's screenshot.

Regenerate current target output before visual capture; stale `.dowe` artifacts do not prove the
contract. If a native simulator is locked or unavailable, mark that target `visualQA: not_run`
and do not report cross-target visual parity as verified.
