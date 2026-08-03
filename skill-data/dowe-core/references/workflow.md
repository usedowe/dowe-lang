# Project workflow

## Small source changes

1. Inspect the owning declaration and every imported binding it uses.
2. Make the smallest source edit with current Dowe syntax.
3. Run the narrowest compiler or target validation.
4. Fix source from the diagnostic; never patch generated `.dowe` output.

## Behavior changes

Use Spec -> Contract -> Tests -> Implementation -> Validation -> Documentation. The project Agent
Harness stores editable plans under `.agents` and generated evidence under
`.dowe/agent-harnesses`. Use its plan, check, status, and validation commands only when the change
needs that workflow.

CodeGraph explains ownership, size, dependencies, and duplication. Use compact context for
orientation and `dowe codegraph check` for declared structural validation. CodeGraph output under
`.dowe/codegraph` is generated evidence; it cannot override a spec or compiler contract.

## Validation choices

| Need | Command family |
| --- | --- |
| Compile or run the project | `dowe dev`, or non-interactive targets such as `dowe dev --target server --target web`, `--target android`, `--target ios` |
| Generate project icons | `dowe icons` or its explicit `--source`, `--background`, `--rounded`, and `--target` options |
| Literal source assertions | `dowe test [path ...]`, with `--json` for stable agent or CI reports |
| Produce production output | `dowe deploy` interactive, `dowe deploy --target static`, `--target docker --registry <registry> --image <name>`, `--target ssh [--publish --host <host> --user <user> [--key-file <path>]]`, `--target cloudflare --name <worker>`, or `dowe deploy web --name <project> --publish` |
| Validate project agent state | Use the Agent Harness check configured under `.agents` |
| Validate a planned feature | Use the selected Harness plan and its declared validation |
| Validate structure | `dowe codegraph check` |

## Native tests

A test file is any `.dowe` file below the project root whose top level contains only `test`
declarations; no reserved directory or filename suffix is required. `dowe test` discovers them
recursively while skipping agent, generated, version-control, vendor, and build trees. Each test has one
quoted name and direct `assert` children comparing parser literals; the runner does not execute
variables, functions, requests, or targets.

```text
test "release metadata"
  assert true value:true
  assert equal actual:{ name:"dowe" channels:["stable", "canary"] } expected:{ name:"dowe" channels:["stable", "canary"] }
```

`assert true` and `assert false` check an exact boolean `value`; `assert equal` compares `actual`
and `expected` structurally. Strings, numbers, booleans, null, barewords, arrays, and objects are
valid literals. For a contract covered by literal values, write the failing test first, run
`dowe test <path>`, implement the smallest compliant change, and run it again. A failed assertion
reports its file, line, and message; no discovered tests is a successful empty run.

Do not start watchers unless the task needs an active development session. Do not read `.env` or
deploy-profile values, serialize server-only bindings into views, or expose Database, KV, HTTP provider,
crypto, or spawn handles to client targets.
