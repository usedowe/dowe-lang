# Growth ladder

Use this recipe for a product story that explains progression, adoption, or utility with a compact
diagram beside explanatory copy.

Selection signals: `growth`, `utility`, `adoption`, `roadmap`, `progress`, `crecimiento`, `barras`,
`escalera`, or `value ladder`.

## Visual contract

- Keep the diagram as one visual payload owned by the growth section.
- Use `Svg` for a static portable chart when its geometry is illustrative. Use a semantic chart
  component when the reference includes interactive or data-driven behavior.
- Keep labels in one stable collection and one `each` template.
- The copy and diagram share a two-column grid at `md` and stack at `xs`.

## Starting source

```dowe
const milestones value:[
  { id:"users" label:"Usuarios" }
  { id:"products" label:"Productos" }
  { id:"transactions" label:"Transacciones" }
  { id:"adoption" label:"Adopción" }
  { id:"value" label:"Valor" }
]

Section id:"growth" background:"meadow" boxed:true
  Grid columns:{ xs:1 md:2 } gap:{ xs:8 md:14 } align:"center"
    Flex direction:"column" align:"start" gap:4 maxW:"xl"
      Text spacing:"widest"
        "UNA ECONOMÍA CON PROPÓSITO"
      Title size:"5xl"
        "Crecimiento impulsado por utilidad."
      Text
        "Cada producto genera demanda real y construye una economía circular para el largo plazo."
      Button href:"/#ecosystem" variant:"ghost" scheme:"secondary"
        "Cómo funciona"
    Grid columns:1 gap:3
      Svg viewBox:"0 0 600 240" w:"full" h:48
        Path d:"M20 210H120V160H20Z" fill:"muted"
        Path d:"M140 210H240V132H140Z" fill:"muted"
        Path d:"M260 210H360V98H260Z" fill:"secondary"
        Path d:"M380 210H480V58H380Z" fill:"secondary"
        Path d:"M500 210H580V20H500Z" fill:"primary"
      Grid columns:5 gap:2
        each in:milestones as:milestone key:milestone.id
          Text size:"sm" align:"center"
            "{milestone.label}"
```

## Refinement checks

Compare the diagram's baseline, bar height progression, label legibility, and alignment with the
copy column. Do not flatten a measured chart into an equal card grid.
