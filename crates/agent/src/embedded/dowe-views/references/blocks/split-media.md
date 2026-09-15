# Split media section

Use this recipe for a section with a bounded explanation, one visual payload, and a compact list of
capabilities or categories. It fits ecosystem, product, services, and feature bands.

Selection signals: `split media`, `feature image`, `ecosystem`, `services`, `producto`, `capabilities`,
`imagen lateral`, `sección dividida`, or `texto e imagen`.

## Visual contract

- `Section` owns the band and `Grid` owns the two responsive tracks.
- Keep the copy in a bounded vertical `Flex` and the media in a `Card` or `Image` owner.
- A list of labels is content and uses one stable collection plus one `each` template.
- At `xs`, copy comes before media unless the reference proves another reading order.
- Do not add a shadow or border to the media frame unless the reference has a contained surface.

## Starting source

```dowe
const capabilities value:[
  { id:"development" label:"Desarrollo" }
  { id:"cloud" label:"Cloud" }
  { id:"marketplace" label:"Marketplace" }
  { id:"payments" label:"Pagos" }
  { id:"services" label:"Servicios" }
  { id:"community" label:"Comunidad" }
]

Section id:"ecosystem" boxed:true
  Grid columns:{ xs:1 md:2 } gap:{ xs:8 md:14 } align:"center"
    Flex direction:"column" align:"start" gap:4 maxW:"xl"
      Text spacing:"widest"
        "ECOSISTEMA"
      Title size:"5xl"
        "Más que una moneda. Un ecosistema."
      Text
        "Conecta desarrollo, cloud, marketplace, pagos y servicios en un mismo entorno."
      Button href:"/#services" variant:"ghost" scheme:"secondary"
        "Explorar el ecosistema"
    Grid columns:1 gap:2
      Card variant:"outlined" scheme:"muted" p:2
        Image src:"/assets/marketing/ecosystem.webp" alt:"Product workspace" aspect:"square" w:"full"
      Flex direction:"column" gap:2
        each in:capabilities as:capability key:capability.id
          Text size:"sm"
            "{capability.label}"
```

## Refinement checks

Compare the two track widths, the media aspect ratio, the copy measure, the list start edge, and the
vertical centerline. If the reference has a full-bleed photograph, move the image to the section
cover contract and add an explicit overlay instead of approximating it with a shadowed card.
