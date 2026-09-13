# Shape a Dowe view

Use this playbook before writing a new screen, landing page, or substantial redesign. It is a
planning reference; it does not authorize source mutation.

1. Identify the visitor mode: Persuade for a landing page, Operate for a task UI, Read for docs,
   or Experience for a portfolio/gallery. Keep the product promise and requested copy fixed.
2. Inspect `main.dowe`, the route graph, the current theme, representative views, and available
   assets. Record what is established and what the request actually changes.
3. Decompose the screen into ordered bands and named regions. For each region record its owner,
   semantic Dowe component, data owner, interaction, responsive rule, accessibility, and media
   payload. Treat repeated same-shape regions as one collection.
4. Write one visual-direction sentence and at most three motifs grounded in the product and
   evidence. Keep each retained band materially distinct; avoid a repeated heading plus equal-card
   template.
5. Select the smallest component tree from the catalog. Mark unknown assets as pending and record
   the independent source needed to resolve them. Do not use a screenshot crop as a substitute.

For a screenshot, continue with `references/reference-ui.md` and create the blueprint before source.
For a page that needs server behavior, load `dowe-server` and create the request-to-route matrix.
