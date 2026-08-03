# `main.dowe` and project structure

## Root files

| Path | Purpose |
| --- | --- |
| `main.dowe` | Required application entrypoint and target wiring |
| `theme.dowe` | Fonts, named color themes, and component visual defaults for views |
| `.env.example` | Shared environment names with empty values or non-secret placeholders |
| `.env` | Local effective values; never read, print, or commit this file |
| `.env.live` | Local Live build and deploy values; never read, print, or commit this file |
| `.env.stage` | Local Stage deploy values; never read, print, or commit this file |
| `.env.uat` | Local UAT deploy values; never read, print, or commit this file |
| `assets/icon.svg` | Recommended transparent vector source for `dowe icons` |
| `icons` | Versioned generated icon sets for web, desktop, iOS, and Android |
| `assets/**` | Public project media served under `/assets/**` in web deployments |

`main.dowe` declares exactly one `main` block. It can own application metadata, one or more view
route graphs, a server, and an optional desktop server.

```text
import apiRoutes from "@/server/endpoints"
import everyDay from "@/server/tasks/every-day"
import viewRoutes from "@/views/routes/view"

main
  app name:"Dowe Journal" bundle:"com.example.dowejournal"
  views:viewRoutes
  server port:8080
    endpoints:apiRoutes
    init
      cron everyDay schedule:"0 3 * * *" args:{}
```

### Main contracts

| Declaration | Accepted props or children |
| --- | --- |
| `app` | `name:string`, `bundle:reverse-DNS string` |
| `views:<binding>` | One imported `views` binding |
| `views:[...]` | Ordered non-empty imported `views` bindings |
| `server` | `port:number`, `endpoints:<binding or list>`; optional `cors`, `init`, inline routes, WebSockets, and protocol children |
| `desktop` | One nested `server` with the same server contract |

`theme.dowe` and `main.dowe` are Dowe configuration roots and cannot be imported. Dotenv files are
not Dowe Source Format and cannot be imported. All reusable modules use static imports and the `@/`
alias for the project root.

## Long declarations

Keep short declarations inline. When props make a line difficult to read, end the declaration header
with `:` and put one prop on each indented line. The header form cannot include inline props.

```text
store session:
  type:SessionState
  persistent:true
  value:{ authorization:"" token:"" user:{ id:"" name:"" email:"" } }
```

The same property-suite form works for other declarations that accept props. Props must appear before
structural children; multiline arrays and objects remain enclosed by `[]` or `{}`.

## Type declarations

`type <Name>` declares a data shape with one `field:type` per indented line. Field types are
`string`, `number`, `bool`, `unknown`, a declared type name, or an array form such as `string[]`
or `Blog[]`. Pure type modules contain only `type` blocks and are importable by both surfaces.

```text
type Blog
  id:string
  title:string
  content:string
```

Views use types with `signal blogs type:Blog[] value:[]` and `store session type:SessionState`.
Server functions use them in `params:{ title:string }` objects and quoted `return` contracts such
as `return:"Blog"`. The compiler validates initial values, binding paths, each-item fields, args,
and returns against the declared shape. Types are compile-time contracts, not view components, and
are never expanded into the view tree.

## Translation catalogs

Optional catalogs live in root `i18n/<locale>.dowe` with two- or three-letter lowercase locale
names. Each file contains one `translations` block; exactly one catalog declares `default:true`.
Nested nodes form dot-separated keys, and a leaf pairs the final segment with quoted text.

```text
translations default:true
  home
    hero
      title "AI GENERATES CODE. DOWE BUILDS SYSTEMS."
  navigation
    views "Views"
```

Every key referenced by a view `i18n` prop must exist in every locale catalog. Missing catalogs,
missing keys, duplicate keys, and invalid locale names fail before target generation. Each target
emits native localization resources; the web runtime falls back from a regional locale such as
`es-CO` to `es`, then to the default locale.

Declare every allowed name in `.env.example` or the selected local file as `NAME=value`. The process
environment overrides `.env` during `dowe dev`, `.env.live` during build or Live deploy, and the
selected `.env.stage` or `.env.uat` during non-Live deploy. The local profiles never fall back to
each other. `.env.example` values are examples and never
become effective values. Dowe source uses static references such as `env.BACKEND_URL`. A name
referenced from views becomes public client configuration, while names used only by server remain
private.

## Example tree

```text
server/
  config/
  handlers/
  middlewares/
  migrations/
  providers/
  types/
  services/
  repositories/
  tasks/
  utils/
  endpoints.dowe
views/
  components/
  layouts/
  pages/
  routes/
  store/
  types/
i18n/
  en.dowe
  es.dowe
assets/
  icon.svg
  icons/
  social/
main.dowe
theme.dowe
.env.example
.env
.env.live
.env.stage
.env.uat
```

This tree is the canonical organization for new source and generated examples, not a parser
restriction for existing projects. Imports remain the compiler authority. Its folder
responsibilities are:

- `server/config` exports reusable `db` and `kv` handle bindings.
- `server/handlers` owns HTTP request parsing and responses.
- `server/middlewares` owns authorization and request context.
- `server/migrations` owns provider-specific server schema migrations.
- `server/types` owns data shapes used only by backend source.
- `server/providers` owns external provider calls.
- `server/services` coordinates business behavior.
- `server/repositories` owns Database and KV logic.
- `server/tasks` owns functions targeted by named `task` or `cron` registrations.
- `server/utils` owns small reusable server transformations.
- `server/endpoints.dowe` connects handlers and middleware to routes.
- `types` owns shared declared data shapes.
- `views/store` owns shared View Stores.
- `views/components` owns reusable view trees.
- `views/layouts` owns application shells.
- `views/pages` owns routed screens.
- `views/routes` connects layouts and pages to route paths.
- `views/types` owns data shapes used only by frontend source.
- `i18n/<locale>.dowe` owns optional translation catalogs; exactly one catalog is default.

Pure types imported by both surfaces may live in a root `types` folder. Do not place a type there
when it belongs exclusively to either `views/types` or `server/types`.

Run `dowe icons` to generate the versioned `icons` tree from a local SVG. Keep that tree in
source control and treat copies under `.dowe` as disposable generated output. Web packages expose
only the synchronized `icons/web` set under `/icons/**`; native icon targets are not public assets.

Web exports preserve the complete public `assets/**` tree under `/assets/**`, including files used
only by document metadata. A source file such as `assets/social/home.png` is therefore available as
`/assets/social/home.png` after static, Cloudflare Worker, or Cloudflare Pages deployment.

The responsibility names `services`, `repositories`, `providers`, `tasks`, and `utils` are optional
folders, not declaration keywords. Their files declare `fn <binding>` because `fn` is the only
reusable server function declaration. `main.dowe`, `theme.dowe`, and optional
`i18n/<locale>.dowe` catalogs have fixed root locations; all other module surfaces are classified
from declarations and imports.
