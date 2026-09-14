# Resolve visual assets

Classify every visual region before authoring: interface, photograph, illustration, icon/logo,
chart, texture, or decoration. Rebuild interface, charts, icons, logos, and controls with Dowe
components. Use `Image` or `cover` for an independently obtained photograph, illustration,
texture, device artwork, or authentic screenshot that has its own provenance.

Search the project `assets/**` tree first and verify exact case. A source file at
`assets/<relative-path>` is referenced as `/assets/<relative-path>` in a view:
`Image src:"/assets/img/hero.webp"` or `cover:"/assets/img/hero.webp"`. Never emit an absolute
filesystem path, a source-relative path, or a reference crop. Keep the same root-relative URL for
all targets; packaging resolves the local file.

If an independent asset is missing, record `status:"missing"` in the visual blueprint and keep the
intended stable path. When image generation is available and authorized, generate only the
independent media, write it under `assets/`, inspect it, and continue the component tree. Legacy
`public/assets/` destinations are normalized to the canonical `assets/` tree. Generation must not
flatten or replace the UI. Do not substitute a generic icon,
SVG, Canvas sketch, gradient, or empty Box for a missing focal illustration.

Record for each asset its path, status, independent source, dimensions/ratio, intended crop or
cover behavior, alt text, and whether it is foreground media or a background owned by a Section,
Card, or media stage. Pending assets are visible QA findings, never visual parity.
