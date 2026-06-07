# Dowe Examples

This directory contains hand-authored Dowe example projects used as a seed corpus for future model-training dataset generation.

The examples are intentionally separate from `src`. They should demonstrate realistic Dowe applications with elegant UI structure, declared routes, endpoint handlers, Store-backed CRUD flows, and current Dowe Source Format conventions.

## Projects

| Project | Focus |
| --- | --- |
| `clinic-desk` | Appointment operations dashboard with Store-backed scheduling endpoints |
| `commerce-ops` | Product inventory console with category and status controls |
| `support-console` | Support ticket queue with priority and lifecycle endpoints |

## Corpus Rules

- Do not copy `src` into examples.
- Keep identifiers and file names in English.
- Use current Dowe Source Format syntax from specs and docs.
- Prefer complete multi-file applications over isolated snippets.
- Keep examples deterministic and provider-independent.

