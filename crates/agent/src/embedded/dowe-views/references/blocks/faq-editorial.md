# Editorial FAQ

Use this recipe when the reference ends with a quiet question list and a link to the complete help
surface. `Accordion` owns disclosure behavior; do not encode arrows or plus signs as text.

Selection signals: `FAQ`, `frequently asked`, `questions`, `accordion`, `preguntas frecuentes`,
`dudas`, or `help list`.

## Visual contract

- Keep the heading and complete-question link in one aligned `Flex` or two-column `Grid`.
- Use one `Accordion` with stable item ids and one direct item per question.
- Use the default quiet treatment when the reference is line-led. Add a scheme or variant only when
  the surface proves a different treatment.
- Preserve question order and visible count from the reference.

## Starting source

```dowe
Section id:"faq" boxed:true
  Flex direction:"column" gap:5
    Flex direction:{ xs:"column" md:"row" } justify:"between" align:{ xs:"start" md:"end" } gap:4
      Flex direction:"column" gap:2
        Text spacing:"widest"
          "PREGUNTAS FRECUENTES"
        Title size:"4xl"
          "Lo esencial, claro."
      Button href:"/faq" variant:"ghost" scheme:"secondary"
        "Ver todas las preguntas"
    Accordion
      item id:"what" label:"¿Qué es el producto?"
        Text
          "Es una economía digital diseñada para la utilidad."
      item id:"how" label:"¿Cómo funciona la protección inicial?"
        Text
          "La protección se define por el contrato vigente del producto."
      item id:"when" label:"¿Cuándo podré usarlo?"
        Text
          "La disponibilidad depende de cada servicio del ecosistema."
      item id:"where" label:"¿Existe un listado público?"
        Text
          "Las condiciones y canales se publican cuando están disponibles."
```

`Accordion` owns structural `item` entries, so its fixed visible questions remain direct children.
Use a data-bound collection for a custom repeated FAQ card or another component that accepts normal
view children; do not invent a dynamic Accordion syntax in a recipe.

## Refinement checks

Compare row height, separator rhythm, question measure, disclosure alignment, and the heading/link
baseline. Keep answer copy out of the initial visual comparison when the reference shows closed rows.
