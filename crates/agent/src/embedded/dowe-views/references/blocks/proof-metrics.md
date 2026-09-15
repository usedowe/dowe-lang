# Compact proof metrics

Use this recipe for the small metric group that supports a hero or introduces a product section.
It preserves the reference's compact evidence block instead of turning each metric into a large
equal-height card across the viewport.

Selection signals: `metrics`, `proof`, `stats`, `numbers`, `indicadores`, `prueba`, `datos`, or
`métricas compactas`.

## Visual contract

- The page or owning section supplies one immutable collection with stable string ids.
- Use one `each` boundary around the complete metric unit.
- Keep the metric value and label in a vertical `Flex`; do not center unrelated metrics through a
  full-width distribution unless the reference does.
- Use a responsive two-column mobile grid and a three-column desktop grid for three records.
- A `Divider` between values belongs inside the metric unit only when the reference shows it.

## Starting source

```dowe
const metrics value:[
  { id:"users" value:"120K" label:"Usuarios activos" }
  { id:"products" value:"48" label:"Productos en uso" }
  { id:"adoption" value:"30%" label:"Adopción inicial" }
]

Grid columns:{ xs:2 md:3 } gap:{ xs:4 md:8 } maxW:"4xl"
  each in:metrics as:metric key:metric.id
    Flex direction:"column" gap:1
      Title size:"3xl"
        "{metric.value}"
      Text size:"sm"
        "{metric.label}"
```

## Refinement checks

Measure the metric group's left edge against the copy column, its total width, the gap between
records, and the distance from the action row. If the reference uses vertical rules, add them to
the metric owner after the grid geometry is correct.
