# Contact information panel

Use this recipe for a contact band that combines a short invitation with a small set of contact
channels. It is useful when the reference has contact content without a visible form.

Selection signals: `contact`, `reach out`, `channels`, `social`, `contacto`, `habla con nosotros`,
`canales`, or `footer contact`.

## Visual contract

- Keep the invitation and channels in separate tracks on desktop and stack them on mobile.
- Give each channel a stable id and render the complete repeated unit through one `each` template.
- Use a semantic `Button` for the primary route; do not style a text line as a fake control.
- Add a form only when the reference shows editable fields and the project has a real request
  contract. A contact panel is not permission to invent a backend.

## Starting source

```dowe
const channels value:[
  { id:"general" label:"General" value:"hello@example.com" }
  { id:"community" label:"Comunidad" value:"/community" }
  { id:"updates" label:"Actualizaciones" value:"/updates" }
]

Section id:"contact" background:"ocean" boxed:true
  Grid columns:{ xs:1 md:2 } gap:{ xs:8 md:14 } align:"center"
    Flex direction:"column" align:"start" gap:3 maxW:"xl"
      Text spacing:"widest"
        "HABLEMOS"
      Title size:"5xl"
        "La próxima etapa empieza aquí."
      Text
        "Conoce el proyecto, revisa la documentación y encuentra el canal adecuado para continuar."
      Button href:"/contact" size:"lg"
        "Contactar"
    Flex direction:"column" gap:3
      each in:channels as:channel key:channel.id
        Flex direction:"column" gap:1
          Text size:"sm"
            "{channel.label}"
          Text size:"lg"
            "{channel.value}"
```

## Refinement checks

Compare the contact copy measure, the first channel alignment, the vertical rhythm between records,
and the primary action's relation to the section rail. Keep the panel readable at narrow widths.
