# Editorial split hero

Use this recipe when the reference has a clear promise on the left, a dominant focal object on the
right, and proof or actions aligned to the same content rail. It is the closest initial match for a
product landing hero such as a coin, device, or platform mark staged against open space.

Selection signals: `hero`, `landing`, `marketing`, `producto`, `moneda`, `object on the right`,
`focal media`, `hero dividido`, or `hero editorial`.

## Visual contract

- Keep the text column bounded with `maxW` so the title wraps at the intended measure.
- Keep the focal media in its own semantic media owner. Use an independently supplied raster
  `Image` for photography or a static `Svg` only when the reference is vector geometry.
- Keep the actions in one `Flex` with a deliberate `gap`; do not distribute buttons across the
  entire grid.
- Put proof records in one `Grid` and one `each` template. The proof grid may remain under the copy
  column when the reference keeps it compact.
- At `xs`, stack the columns and preserve the focal media after the promise and actions. The exact
  asset crop is an asset decision, not a reason to add arbitrary offsets.

## Starting source

Replace the asset path with a project-owned or independently obtained asset before capture.

```dowe
const heroProof value:[
  { id:"supply" value:"1B" label:"Supply máximo" }
  { id:"price" value:"US$0,01" label:"Precio Genesis" }
  { id:"protection" value:"30%" label:"Protección inicial" }
]

Section id:"hero" background:"ocean" boxed:true
  Grid columns:{ xs:1 md:2 } gap:{ xs:10 md:16 } align:"center"
    Flex direction:"column" align:"start" gap:4 maxW:"xl"
      Text spacing:"widest"
        "UNA MONEDA PARA UN MUNDO REAL"
      Title as:"h1" size:"6xl"
        "Tecnología que genera valor real."
      Text size:"lg"
        "Una economía digital diseñada para la utilidad, la circulación y el crecimiento sostenible."
      Flex gap:3
        Button href:"/#genesis" size:"lg"
          "Invertir en Genesis"
        Button href:"/#ecosystem" variant:"outlined" scheme:"muted" size:"lg"
          "Conocer más"
      Grid columns:{ xs:2 md:3 } gap:4 w:"full"
        each in:heroProof as:proof key:proof.id
          Flex direction:"column" gap:1
            Title size:"3xl"
              "{proof.value}"
            Text size:"sm"
              "{proof.label}"
    Card variant:"ghost" scheme:"background" p:{ xs:4 md:6 } minH:{ xs:80 md:96 }
      Image src:"/assets/marketing/hero-focal.webp" alt:"Focal product composition" aspect:"square" w:"full" h:"full"
```

## Refinement checks

Compare the text column width, title line count, media subject position, media crop, action row
alignment, and proof grid width as separate regions. Fix the grid and semantic owners first; only
then adjust a scalar size or an intentional asset crop.
