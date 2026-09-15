# Dark opportunity band

Use this recipe for a high-contrast invitation band with a short message, one action, and a strong
background or foreground visual. It creates a real change in section rhythm between light marketing
bands.

Selection signals: `opportunity`, `genesis`, `dark band`, `dark section`, `launch`, `invitación`,
`banda oscura`, or `high contrast CTA`.

## Visual contract

- Keep the full-width tonal change on `Section`; keep the readable content surface on the semantic
  `Card` only when the reference contains a distinct panel.
- Use a two-track `Grid` for message and supporting visual detail.
- Preserve the supporting visual as `Image` or `Svg` with independent provenance. Do not use a
  screenshot crop as the background.
- Give the band a real minimum height only when its visual scene needs one; do not compress a
  photographic subject into a shallow strip.

## Starting source

```dowe
Section id:"genesis" background:"slate"
  Card scheme:"primary" rounded:"lg" minH:{ xs:72 md:96 } p:{ xs:7 md:10 }
    Grid columns:{ xs:1 md:2 } gap:{ xs:8 md:12 } align:"center"
      Flex direction:"column" align:"start" gap:3 maxW:"2xl"
        Text spacing:"widest"
          "OPORTUNIDAD"
        Title size:"5xl"
          "Sé parte del comienzo."
        Text
          "Una oportunidad limitada para adquirir el producto en las mejores condiciones."
        Button href:"/genesis" scheme:"secondary" size:"lg"
          "Invertir en Genesis"
      Svg viewBox:"0 0 520 260" w:"full" h:48
        Path d:"M24 226H496V242H24Z" fill:"secondary"
        Path d:"M88 226L170 92L246 226Z" fill:"primary"
        Path d:"M170 226L282 42L404 226Z" fill:"secondary"
        Path d:"M310 226L374 122L470 226Z" fill:"primary"
```

## Refinement checks

Compare band height, panel inset, message width, action baseline, and the supporting scene's visual
weight. The section should feel intentionally darker and denser without making the content unreadable.
