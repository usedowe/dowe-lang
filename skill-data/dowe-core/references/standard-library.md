# Standard library reference

The standard library is a fixed catalog of namespaced portable functions validated by the
compiler; it is not a package import system. Server statements bind results with
`<namespace> <binding> source:"<function>" <props>`. View functions assign results with
`set <target> source:<namespace>.<function> <props>`.

```text
str name source:"trim" value:body.name
```

```text
set cleanedName source:str.trim value:form.name
```

Functions are pure except `date.now`. Results are JSON-compatible; invalid parse inputs return
`fallback` or `null`; numeric results must be finite and division by zero returns `null`; `sort`
and `list` return new values without mutating the source. Standard-library functions never open
network connections, read files, spawn processes, read environment variables, or access Database,
Cache, or Vector. `id <binding> source:"ulid"` is server-only.

## str

| Function | Required args | Optional args | Result |
| --- | --- | --- | --- |
| `str.trim`, `str.lower`, `str.upper` | `value` | | string |
| `str.length` | `value` | | number of Unicode scalars |
| `str.contains` | `value`, `needle` | | bool |
| `str.startsWith` | `value`, `prefix` | | bool |
| `str.endsWith` | `value`, `suffix` | | bool |
| `str.replace` | `value`, `from`, `to` | | string |
| `str.split` | `value`, `delimiter` | `limit` | array |
| `str.join` | `values` | `delimiter` | string |

## math

| Function | Required args | Result |
| --- | --- | --- |
| `math.add`, `math.sub`, `math.mul` | `left`, `right` | number |
| `math.div` | `left`, `right` | number or `null` |
| `math.round`, `math.floor`, `math.ceil`, `math.abs` | `value` | number |
| `math.min`, `math.max`, `math.average` | `values` | number or `null` on empty arrays |
| `math.sum` | `values` | number; `0` on empty arrays |

## parse

| Function | Required args | Optional args | Result |
| --- | --- | --- | --- |
| `parse.int` | `value` | `fallback` | number, fallback, or `null`; rejects decimals |
| `parse.float` | `value` | `fallback` | finite number, fallback, or `null` |
| `parse.bool` | `value` | `fallback` | bool, fallback, or `null` |
| `parse.json` | `value` | `fallback` | JSON value, fallback, or `null` |
| `parse.string` | `value` | `fallback` | string |
| `parse.svg` | `value` | `fallback` | Dowe `Svg`/`Path` source text, fallback, or `null` |

`parse.svg` accepts bounded SVG XML (`svg`, `g`, `path`, inline fills, `matrix(...)` transforms, at
most 262144 UTF-8 bytes and 1024 paths), maps external colors to semantic tokens, and never
executes DTD, entities, scripts, or external resources.

## url

| Function | Required args | Optional args | Result |
| --- | --- | --- | --- |
| `url.encode` | `value` | | string |
| `url.decode` | `value` | `fallback` | string, fallback, or `null` |
| `url.parse` | `value` | | object with `ok`, `scheme`, `host`, `path`, `query`, `fragment`, `origin`, `isRelative`, `error` |
| `url.queryGet` | `value`, `name` | | string or `null` |
| `url.querySet` | `value`, `name`, `param` | | string |

## csv

| Function | Required args | Optional args | Result |
| --- | --- | --- | --- |
| `csv.parse` | `value` | `delimiter`, `header`, `maxRows`, `maxColumns` | object with `columns`, `rows`, `errors`, `truncated`, `rowCount` |
| `csv.stringify` | `rows` | `delimiter` | string |

## sort

| Function | Required args | Optional args | Result |
| --- | --- | --- | --- |
| `sort.asc`, `sort.desc` | `values` | | stable new array |
| `sort.by` | `values`, `field` | `direction`, `nulls` | stable new array |

## list

| Function | Required args | Result |
| --- | --- | --- |
| `list.take`, `list.skip` | `values`, `count` | array |
| `list.first`, `list.last` | `values` | value or `null` |
| `list.count` | `values` | number |
| `list.filterEquals`, `list.filterContains` | `values`, `field`, `value` | array |
| `list.mapField` | `values`, `field` | array |
| `list.sumBy` | `values`, `field` | number |
| `list.averageBy` | `values`, `field` | number or `null` |

Field paths use dot notation against JSON-compatible objects.

## json

| Function | Required args | Optional args | Result |
| --- | --- | --- | --- |
| `json.get` | `value`, `path` | `fallback` | value, fallback, or `null` |
| `json.set` | `value`, `path`, `next` | | object |
| `json.pick`, `json.omit` | `value`, `fields` | | object |
| `json.merge` | `left`, `right` | | shallow-merged object |
| `json.stringify` | `value` | `pretty` | string |
| `json.parse` | `value` | `fallback` | JSON value, fallback, or `null` |

## date

| Function | Required args | Result |
| --- | --- | --- |
| `date.now` | | ISO UTC string from the target clock |
| `date.formatIso` | `value` | ISO UTC string or the original string |
| `date.addDays` | `value`, `days` | ISO UTC string or `null` |
| `date.diffDays` | `start`, `end` | number |
