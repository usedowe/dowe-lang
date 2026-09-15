# Media CTA panel

Use this recipe for a closing invitation that combines a clear message, two actions, and a
supporting image or product scene inside a contained panel.

Selection signals: `CTA`, `call to action`, `white paper`, `build together`, `media panel`,
`llamada a la acción`, `panel final`, or `construyamos juntos`.

## Visual contract

- Use one contained `Card` inside a section with a distinct but related background.
- Keep actions in one `Flex` and preserve their primary/secondary relationship.
- The media remains a separate `Image` or `Svg` region and may occupy the entire second track.
- A short closing panel can be visually rich without becoming a second hero; keep its copy measure
  and title scale below the hero values.

## Starting source

```dowe
Section id:"future" boxed:true
  Card scheme:"primary" rounded:"lg" p:{ xs:7 md:10 } shadow:"lg"
    Grid columns:{ xs:1 md:2 } gap:{ xs:8 md:12 } align:"center"
      Flex direction:"column" align:"start" gap:3 maxW:"xl"
        Text spacing:"widest"
          "EL FUTURO ES ABIERTO"
        Title size:"4xl"
          "Construyamos juntos"
        Text
          "Una visión clara para que la tecnología genere más oportunidades para las personas."
        Flex gap:3
          Button href:"/genesis" scheme:"secondary"
            "Invertir en Genesis"
          Button href:"/white-paper" variant:"outlined" scheme:"secondary"
            "Leer White Paper"
      Image src:"/assets/marketing/future-mountains.webp" alt:"Product future landscape" aspect:"horizontal" w:"full"
```

## Refinement checks

Compare the panel inset, action widths, media crop, text contrast, and the visual balance between
copy and image. Ensure the media does not push the action row below the intended panel fold.
