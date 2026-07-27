# Font Assets

This directory contains the local font assets used by the Dowe font catalog.

The source of truth for the catalog is `crates/components/src/component/font_catalog.rs`. Each packaged catalog entry must have a matching directory named after the font token, a `LICENSE.txt`, and one `.ttf` file for every distinct `asset_stem` declared by the catalog.

`dowe dev` does not copy every catalog family. The effective generated set is `dowe.json` `fonts.default`, `dowe.json` `fonts.install`, and any family referenced by a view `font` prop. That set is copied first into `.dowe/fonts` so web, iOS, and Android can reuse the same generated pool.

## Layout

| Path | Purpose |
| --- | --- |
| `assets/fonts/inter/*.ttf` | Inter static weight instances |
| `assets/fonts/roboto/*.ttf` | Roboto static weight instances |
| `assets/fonts/montserrat/*.ttf` | Montserrat static weight instances |
| `assets/fonts/lato/*.ttf` | Lato static weight files |
| `assets/fonts/poppins/*.ttf` | Poppins static weight files |
| `assets/fonts/manrope/*.ttf` | Manrope static weight instances |
| `assets/fonts/quicksand/*.ttf` | Quicksand static weight instances |
| `assets/fonts/lora/*.ttf` | Lora static weight instances |

## Weights

Each packaged family stores static files for the common shipped faces and maps the complete Dowe weight scale to those assets deterministically:

| Dowe weight | CSS/native weight | Asset suffix |
| --- | --- | --- |
| `thin` | `100` | `light` |
| `extralight` | `200` | `light` |
| `light` | `300` | `light` |
| `regular` | `400` | `regular` |
| `medium` | `500` | `medium` |
| `semibold` | `600` | `semibold` |
| `bold` | `700` | `bold` |
| `extrabold` | `800` | `extrabold` |
| `black` | `900` | `extrabold` |

Inter, Roboto, Montserrat, Manrope, Quicksand, and Lora are generated as static instances from their upstream variable fonts. Lato and Poppins use upstream static `.ttf` files. All files were sourced from the Google Fonts repository and keep the upstream font licenses in each family directory.
