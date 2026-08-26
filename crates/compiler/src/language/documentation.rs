use crate::language::completion::{component_value_completions, props_for_component};
use dowe_components::{BuiltinComponent, ColorFamily};
use dowe_stdlib::{StdlibReturnKind, StdlibSignature};

#[rustfmt::skip]
pub(super) const VIEW_COMPONENTS: &[&str] = &[
    "Box", "Section", "Flex", "Grid", "Input", "Select", "Option", "Code", "Video", "Iframe", "Device", "Canvas",
    "Candlestick", "ArcChart", "AreaChart", "BarChart", "LineChart", "PieChart", "Table", "Divider",
    "Button", "Brand", "Banner", "ToggleTheme", "SelectTheme", "Fab", "fabAction", "Slider", "Dropzone", "ComboBox",
    "comboOption", "CsvField", "csvColumn", "DragDrop", "dragGroup", "dragItem", "Editor", "ImageCropper",
    "Password", "Phone", "Pin", "Textarea", "Alert", "Icon", "Svg", "Path", "AppBar", "Footer",
    "BottomBar", "NavMenu", "SideNav", "RailNav", "Sidebar", "Scaffold", "Splash", "Drawer", "Avatar", "Badge", "Chip",
    "Skeleton", "Modal", "AlertDialog", "Tooltip", "Toast", "Dropdown", "Command", "AvatarGroup", "ChatBox",
    "Empty", "Marquee", "TypeWriter", "RichText", "Record", "ToggleGroup", "Collapsible", "Countdown", "Map",
    "Audio", "Camera", "Microphone", "Image", "Accordion", "Carousel", "Checkbox", "Color", "Date", "DateRange", "RadioGroup", "Toggle",
    "Card", "Tabs", "tab", "Stepper", "step", "Title", "Text",
];

struct ServerDocumentation {
    name: &'static str,
    signature: &'static str,
    description: &'static str,
}

const SERVER_DOCUMENTATION: &[ServerDocumentation] = &[
    ServerDocumentation {
        name: "main",
        signature: "main",
        description: "Declares the project entrypoint and its optional app, server, desktop server, and one-or-many imported views capabilities.",
    },
    ServerDocumentation {
        name: "server",
        signature: "server port:<number> [endpoints:<symbol|array>] [databases:<symbol|array>]",
        description: "Declares a Rust-backed Dowe server target with optional imported endpoint groups and Database handles registered for project operations.",
    },
    ServerDocumentation {
        name: "databases",
        signature: "databases:[<database-binding>...]",
        description: "Registers imported server-only Database handles so migrations, seeders, and runtime preparation can discover them from main.dowe.",
    },
    ServerDocumentation {
        name: "tls",
        signature: "tls mode:\"acme|local\" domains:<array> [email:<string>] [staging:<boolean>] [cache:<string>] [domainsFrom:<object>] [refreshSeconds:<number>] [httpPort:<number>]",
        description: "Terminates HTTPS in the Rust server, reloads managed domain catalogs, and can redirect authorized HTTP hosts.",
    },
    ServerDocumentation {
        name: "endpoints",
        signature: "endpoints <name>",
        description: "Exports a named server endpoint graph with one-level groups that main can reference alone or in a list.",
    },
    ServerDocumentation {
        name: "route",
        signature: "route \"/...\" [middleware:<reference|array>]",
        description: "Declares an inline HTTP route, static response, handler, or method map under a server block.",
    },
    ServerDocumentation {
        name: "method",
        signature: "method <HTTP_METHOD> handler:<handler>",
        description: "Maps an inline server route method to a handler.",
    },
    ServerDocumentation {
        name: "get",
        signature: "get path:\"/...\" handler:<handler>",
        description: "Declares a GET endpoint inside an imported endpoint group.",
    },
    ServerDocumentation {
        name: "post",
        signature: "post path:\"/...\" handler:<handler>",
        description: "Declares a POST endpoint inside an imported endpoint group.",
    },
    ServerDocumentation {
        name: "put",
        signature: "put path:\"/...\" handler:<handler>",
        description: "Declares a PUT endpoint inside an imported endpoint group.",
    },
    ServerDocumentation {
        name: "patch",
        signature: "patch path:\"/...\" handler:<handler>",
        description: "Declares a PATCH endpoint inside an imported endpoint group.",
    },
    ServerDocumentation {
        name: "delete",
        signature: "delete path:\"/...\" handler:<handler>",
        description: "Declares a DELETE endpoint inside an imported endpoint group.",
    },
    ServerDocumentation {
        name: "handler",
        signature: "handler <name>",
        description: "Declares an asynchronous server request handler with implicit `req`; do not add `async` or `await`.",
    },
    ServerDocumentation {
        name: "middleware",
        signature: "middleware <name> [params:{ ... }]",
        description: "Declares asynchronous request middleware with implicit `req` and explicit `next` continuation.",
    },
    ServerDocumentation {
        name: "fn",
        signature: "fn <name> [params:{ name:Type }] [return:\"Type\"]",
        description: "Declares a reusable typed server function; invoke an imported function with `<name> <result> args:{ ... }`.",
    },
    ServerDocumentation {
        name: "database",
        signature: "database <binding> provider:\"postgres|d1|dowe\" host:<value> port:<value> account:<value> secret:<value> name:\"name\" entities:[...] seeders:[...]",
        description: "Declares a server-only Database connection that uses local Dowe persistence during development.",
    },
    ServerDocumentation {
        name: "entity",
        signature: "entity <name>",
        description: "Declares an importable Database entity and its typed fields.",
    },
    ServerDocumentation {
        name: "seeder",
        signature: "seeder <name>",
        description: "Declares importable static Database seed inserts.",
    },
    ServerDocumentation {
        name: "insert",
        signature: "insert entity:<entity> value:{ ... }",
        description: "Adds one static entity record to an importable Database seeder.",
    },
    ServerDocumentation {
        name: "query",
        signature: "query <binding> conn:<handle>.<operation> ...",
        description: "Runs a Database operation and declares its result binding.",
    },
    ServerDocumentation {
        name: "cache",
        signature: "cache <binding> provider:\"kv|redis|dowe\"|env.NAME host:<value> port:<value> account:<value> secret:<value> name:<value>",
        description: "Declares a server-only Cache connection that uses local Dowe persistence during development.",
    },
    ServerDocumentation {
        name: "kv",
        signature: "kv <binding> conn:<cache>.<get|set|delete|keys|clear> ...",
        description: "Runs a Cache key-value operation and declares its result binding.",
    },
    ServerDocumentation {
        name: "vector",
        signature: "vector <binding> provider:\"dowe\" host:<value> port:<value> account:<value> secret:<value> name:<value>",
        description: "Declares a server-only Vector database that is embedded during development and can be local or WebSocket-backed in production.",
    },
    ServerDocumentation {
        name: "queue",
        signature: "queue service | queue <binding> provider:\"dowe|rabbitmq|cloudflare|vercel\" host:<value> port:<value> account:<value> secret:<value> vhost:<value>",
        description: "Hosts the authenticated Dowe Queue WebSocket service or declares a server-only Queue connection for Dowe, RabbitMQ, Cloudflare, or Vercel.",
    },
    ServerDocumentation {
        name: "msg",
        signature: "msg <binding> conn:<queue>.publish queue:<value> payload:<json>",
        description: "Directly publishes JSON to an already declared durable Queue and returns `{ ok, id }` after a durable or confirmed enqueue.",
    },
    ServerDocumentation {
        name: "emb",
        signature: "emb <binding> conn:<vector>.<upsert|search|read|delete|list> ...",
        description: "Stores, searches, reads, deletes, or lists embeddings through a Vector connection.",
    },
    ServerDocumentation {
        name: "websocket",
        signature: "websocket path:\"/...\" [middleware:<reference|array>]",
        description: "Declares a WebSocket route with optional middleware and open, message, close, and drain handlers.",
    },
    ServerDocumentation {
        name: "udp",
        signature: "udp name:\"...\" [bind:\"...\"] port:<number>",
        description: "Declares a UDP transport handled by the Rust server runtime.",
    },
    ServerDocumentation {
        name: "tcp",
        signature: "tcp name:\"...\" [bind:\"...\"] port:<number>",
        description: "Declares a TCP transport handled by the Rust server runtime.",
    },
    ServerDocumentation {
        name: "rtp",
        signature: "rtp [bind:\"...\"] min:<port> max:<port>",
        description: "Declares an RTP transport and its validated media configuration.",
    },
    ServerDocumentation {
        name: "model",
        signature: "model name:\"...\" kind:\"...\" engine:\"...\" format:\"...\" [source:\"...\"] [sampleRates:<array>]",
        description: "Declares a server-owned model resource.",
    },
    ServerDocumentation {
        name: "cors",
        signature: "cors origins:[...] methods:[...] headers:[...]",
        description: "Configures validated CORS behavior for the server.",
    },
    ServerDocumentation {
        name: "init",
        signature: "init",
        description: "Runs server startup statements before traffic, or an unnamed page/layout workflow once when that view scope mounts.",
    },
    ServerDocumentation {
        name: "redirect",
        signature: "redirect path:\"/...\"",
        description: "Replaces the active internal route and terminates the current view fn or init workflow.",
    },
    ServerDocumentation {
        name: "response",
        signature: "response [status:<number>] [json:<value>|text:<string>]",
        description: "Declares a static HTTP route response outside a handler.",
    },
    ServerDocumentation {
        name: "return",
        signature: "return [status:<number>] json:<value>|text:<string>|bytes:<binding>|proxy:<binding>|reverse:<cacheBinding.url> [strategy:\"roundRobin\" state:<cacheBinding.state> loadingUrl:<cacheBinding.url> errorUrl:<cacheBinding.url>]|agent:<binding>",
        description: "Returns an HTTP response directly from a handler or middleware; server functions use `return value:<value>`.",
    },
    ServerDocumentation {
        name: "str",
        signature: "str <binding> source:\"<function>\" <props>",
        description: "Runs a String standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "math",
        signature: "math <binding> source:\"<function>\" <props>",
        description: "Runs a Math standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "parse",
        signature: "parse <binding> source:\"<function>\" <props>",
        description: "Runs a Parse standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "url",
        signature: "url <binding> source:\"<function>\" <props>",
        description: "Runs a URL standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "csv",
        signature: "csv <binding> source:\"<function>\" <props>",
        description: "Runs a CSV standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "sort",
        signature: "sort <binding> source:\"<function>\" <props>",
        description: "Runs a Sort standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "list",
        signature: "list <binding> source:\"<function>\" <props>",
        description: "Runs a List standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "json",
        signature: "json <binding> source:\"<function>\" <props>",
        description: "Runs a JSON standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "date",
        signature: "date <binding> source:\"<function>\" <props>",
        description: "Runs a Date standard-library function and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "id",
        signature: "id <binding> source:\"ulid\"",
        description: "Generates a server-only identifier and declares its result binding without assignment syntax.",
    },
    ServerDocumentation {
        name: "jwt",
        signature: "jwt <binding> secret:<env> algorithm:\"HS256\" token|claims:<value>",
        description: "Binds a server-only JWS or JWE result directly in the current action or middleware.",
    },
    ServerDocumentation {
        name: "if",
        signature: "if <condition>",
        description: "Conditionally evaluates an indented Dowe block.",
    },
    ServerDocumentation {
        name: "next",
        signature: "next [context:{ ... }]",
        description: "Allows middleware processing to continue to the next stage.",
    },
    ServerDocumentation {
        name: "log",
        signature: "log <values...>",
        description: "Writes a standard server log event.",
    },
    ServerDocumentation {
        name: "info",
        signature: "info <values...>",
        description: "Writes an informational server log event.",
    },
    ServerDocumentation {
        name: "warn",
        signature: "warn <values...>",
        description: "Writes a warning server log event.",
    },
    ServerDocumentation {
        name: "error",
        signature: "error <values...>",
        description: "Writes an error server log event.",
    },
    ServerDocumentation {
        name: "task",
        signature: "task fn:<fn> [args:{ ... }] [after:\"headers\"] | task [args:{ ... }] [after:\"headers\"] <server statements...>",
        description: "Starts an imported server function or inline server-function body in an isolated process and discards its result. Tasks are immediate and source-ordered by default; `after:\"headers\"` is valid only directly in a reverse-proxy HTTP handler, requires `args.event` to be an object, and launches once after real upstream response headers arrive.",
    },
    ServerDocumentation {
        name: "cron",
        signature: "cron fn:<fn> schedule:\"<cron>\" [args...]",
        description: "Schedules isolated UTC executions from server init without creating a result binding.",
    },
    ServerDocumentation {
        name: "send",
        signature: "send ws json:<value>",
        description: "Sends a payload through the active supported transport.",
    },
    ServerDocumentation {
        name: "bridge",
        signature: "bridge sse:<reference> to:ws [requestId:<value>] [requestType:<value>] [model:<value>]",
        description: "Bridges a supported server transport to another runtime surface.",
    },
    ServerDocumentation {
        name: "request",
        signature: "request <binding> source:\"query|rawQuery|header|cookie|bytes\" [name:<string>]",
        description: "Reads request metadata or the byte-exact HTTP body into an explicit result binding; header and cookie sources require `name`.",
    },
    ServerDocumentation {
        name: "file",
        signature: "file <binding> source:\"write|read|exists|delete\" root:<path> path:<relative-path> [data:<bytes>]",
        description: "Reads or atomically mutates server-only files confined below a configured storage root.",
    },
    ServerDocumentation {
        name: "password",
        signature: "password <binding> source:\"hash|verify\" value:<password> [hash:<phc>] [required:true]",
        description: "Hashes passwords with salted Argon2id PHC strings or verifies them in the server runtime.",
    },
    ServerDocumentation {
        name: "bearer",
        signature: "bearer <binding> value:req.header.Authorization",
        description: "Extracts a bearer token from a request authorization header.",
    },
    ServerDocumentation {
        name: "http",
        signature: "http <binding> method:\"get|post|put|patch|delete\" base:<url> path:\"/...\" [bearer:<secret>] [headers:<object>] [json:<value>] [mode:\"json|proxy|bytes\"] [redirect:\"follow|manual|error\"] [maxRedirects:<number>] [timeoutMs:<number>]",
        description: "Performs a validated outbound HTTP request.",
    },
    ServerDocumentation {
        name: "agent",
        signature: "agent <binding> source:\"chat\" request:<request>",
        description: "Transforms a server-side Dowe Agent chat request and declares its result binding.",
    },
    ServerDocumentation {
        name: "ai",
        signature: "ai <binding> source:\"chat\" prompt:<value> files:<value> [model:\"...\"]",
        description: "Runs a server-side local AI chat request with project file context.",
    },
    ServerDocumentation {
        name: "ws",
        signature: "ws <binding> source:\"json\"",
        description: "Parses the active WebSocket message as a JSON-compatible result binding.",
    },
    ServerDocumentation {
        name: "spawn",
        signature: "spawn <binding> command:<value> [args:<array>] [cwd:<value>] [timeoutMs:<number>] [maxOutputBytes:<number>] [background:<boolean>]",
        description: "Runs a process through the shared sandboxed Dowe spawn runtime.",
    },
    ServerDocumentation {
        name: "crypto",
        signature: "crypto <binding> encryption:\"aesCtr|cencAesCtr\" data:<reference> key:<value> iv:<value> [subsamples:<value>]",
        description: "Transforms bytes with AES-CTR or CENC AES-CTR in the Rust server runtime.",
    },
    ServerDocumentation {
        name: "commit",
        signature: "commit [value:<value>]",
        description: "Commits the current Store transaction.",
    },
    ServerDocumentation {
        name: "rollback",
        signature: "rollback",
        description: "Rolls back the current Store transaction.",
    },
];

pub(super) fn server_names() -> impl Iterator<Item = &'static str> {
    SERVER_DOCUMENTATION.iter().map(|entry| entry.name)
}

pub(super) fn server_props(name: &str) -> Vec<&'static str> {
    SERVER_DOCUMENTATION
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| {
            entry
                .signature
                .split_whitespace()
                .filter_map(|token| {
                    let token = token.trim_matches(['[', ']']);
                    token.split_once(':').map(|(name, _)| name)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn component_documentation(name: &str) -> Option<String> {
    if name == "validate" {
        return Some(
            "## `validate`\n\nDeclares one ordered client-side validation rule inside `Input`, `Date`, `Pin`, `Phone`, `Select` or `Checkbox`, or validates a Signal globally inside a view function with `validate signalName`. The first failing rule supplies the visible error after the control is touched.\n\n**Accepted props**\n\n- `rule`: quoted rule identifier\n- `message`: quoted non-empty error message"
                .to_string(),
        );
    }
    BuiltinComponent::from_name(name)?;
    let props = props_for_component(name);
    let mut output = format!(
        "## `{name}`\n\n{}\n\n**Accepted props**\n",
        component_description(name)
    );
    for prop in props {
        output.push_str(&format!("\n- `{prop}`: {}", prop_type(name, prop)));
    }
    let children = component_children(name);
    if !children.is_empty() {
        output.push_str("\n\n**Accepted children**\n");
        for (child, description) in children {
            output.push_str(&format!("\n- `{child}` {description}"));
        }
    }
    Some(output)
}

pub(super) fn component_prop_documentation(component: &str, prop: &str) -> Option<String> {
    let props = props_for_component(component);
    (!props.is_empty() && props.contains(&prop)).then(|| {
        let value_type = prop_type(component, prop);
        let description = if component == "Icon" && prop == "name" {
            "Selects a member of the shared Solar, country-flag, SVG Spinner, or SVG Logos catalog. A bare path must resolve to a string; Signal changes update the icon and invalid runtime values use the validated initial icon."
        } else if component == "Image" && prop == "src" {
            "Accepts a quoted packaged asset or HTTPS URL, or a bare path resolving to a string constant, Signal, or each-item value. The compiler validates the path and string type before lowering it for every target."
        } else {
            prop_description(prop)
        };
        format!(
            "### `{component}.{prop}`\n\n**Type:** `{value_type}`\n\n{}",
            description
        )
    })
}

pub(super) fn theme_documentation(owner: &str, token: &str, root_theme: bool) -> Option<String> {
    match (owner, token, root_theme) {
        ("theme", "theme", true) => Some(
            "## `theme`\n\n```dowe\ntheme\n  fonts default:\"inter\" install:[\"inter\"]\n  design defaultTheme:\"light\"\n```\n\nDeclares the canonical project theme configuration in `theme.dowe`.\n\n**Accepted children**\n\n- `app` for visible application metadata\n- `fonts` for the default and installed font families\n- `design` for the default theme, component defaults, and named themes"
                .to_string(),
        ),
        ("theme", "theme", false) => Some(
            "## `theme`\n\n```dowe\ntheme name:\"brand\" extends:\"light\"\n  colors:\n    primary color:\"#2563eb\" text:\"#ffffff\" title:\"#fffffe\"\n```\n\nDeclares a named color theme inside `design`. Each grouped semantic family declares `color`, `text`, and `title`: the base value, ordinary content and controls, and titles and semantic headers respectively. A named theme may inherit from another named or built-in theme and override any role within a family. Component defaults belong in the component entries under `design`; explicit usage props take precedence over those defaults.\n\n**Accepted props**\n\n- `name`: stable lowercase theme name\n- `extends`: optional theme name to inherit\n\n**Accepted children**\n\n- `colors` with grouped semantic color families"
                .to_string(),
        ),
        (owner, token, _)
            if !matches!(owner, "design" | "fonts")
                && ColorFamily::from_theme_name(owner).is_some()
                && owner == token =>
        {
            Some(format!(
                "## `{owner}` color family\n\nDeclares one grouped semantic color family. Its `color`, `text`, and `title` props are normalized into the shared target-neutral color tokens. In an inherited theme, any omitted role comes from the parent theme."
            ))
        }
        (owner, "color", _) if ColorFamily::from_theme_name(owner).is_some() => Some(
            "### `color`\n\nBase semantic color used as the filled surface or family accent."
                .to_string(),
        ),
        (owner, "text", _) if ColorFamily::from_theme_name(owner).is_some() => Some(
            "### `text`\n\nSemantic color for ordinary content, control labels, and authored `Text` inside the family surface."
                .to_string(),
        ),
        (owner, "title", _) if ColorFamily::from_theme_name(owner).is_some() => Some(
            "### `title`\n\nSemantic color for authored `Title` and integrated semantic headers inside the family surface."
                .to_string(),
        ),
        ("design", "design", _) => Some(
            "## `design`\n\n```dowe\ndesign defaultTheme:\"light\"\n  Button variant:\"outlined\"\n  Input variant:\"outlined\" scheme:\"primary\"\n  Text font:\"manrope\"\n  Title font:\"syne\"\n  theme name:\"light\"\n```\n\nConfigures the default named color theme and static visual defaults that Dowe injects into the shared view model. The precedence is explicit usage prop, then the matching `design` entry, then the built-in component default. Built-in defaults intentionally add no border or shadow unless the project configures those props. The normalized defaults are shared by web, desktop, Android, and iOS output.\n\n**Accepted props**\n\n- `defaultTheme`: declared theme name used initially\n\n**Accepted children**\n\n- `Button`, `IconButton`, `Card`, `Drawer`, `Toast`, `Section`, `Accordion`, `Checkbox`, `Input`, `Date`, `Password`, `Select`, `Pin`, `AppBar`, `Footer`, `Modal`, `Dropdown`, `Tooltip`, `Tabs`\n- `Chip`, `SideNav`, `Sidebar`, `NavMenu`, `Avatar`, and `Ui` for existing shared visual defaults\n- `Text` for the default text font\n- `Title` for the default title font\n- `theme` for named color tokens"
                .to_string(),
        ),
        ("Tabs", "Tabs", _) => Some(
            "## `Tabs` theme defaults\n\n```dowe\nTabs variant:\"pills\" scheme:\"primary\"\n```\n\nDeclares optional static defaults for `Tabs` inside `design`. Its `variant` accepts `solid`, `outlined`, `line`, `ghost`, or `pills`; the built-in default is `pills` with the `primary` scheme. An explicit prop on a component usage always wins; omitted props retain Dowe's built-in component defaults."
                .to_string(),
        ),
        (component @ ("Card" | "Button" | "IconButton" | "Drawer" | "Toast" | "Section" | "Accordion" | "Checkbox" | "Input" | "Date" | "Password" | "Select" | "Pin" | "AppBar" | "Footer" | "Modal" | "Dropdown" | "Tooltip" | "Chip" | "Avatar" | "Ui"), token, _)
            if component == token => Some(
            format!(
                "## `{component}` theme defaults\n\n```dowe\n{component} variant:\"outline\" scheme:\"primary\" radius:\"xs\" shadow:\"xs\"\n```\n\nDeclares optional static defaults for `{component}` inside `design`. An explicit prop on a component usage always wins; omitted props retain Dowe's built-in component defaults.\n\n**Accepted props**\n\n- `variant`: `solid`, `outline`, `outlined`, `line`, or `ghost`\n- `scheme`: semantic color family\n- `radius` or `rounded`: `xs`, `sm`, `md`, `lg`, `xl`, or `full`\n- `shadow`: `xs`, `sm`, `md`, `lg`, or `xl`\n- `shadowColor`: semantic color family\n- `border`: integer from `1` to `4`\n- `borderColor`: semantic color family\n- `size`: `xs`, `sm`, `md`, `lg`, or `xl`"
            ),
        ),
        (component @ ("Text" | "Title"), token, _) if component == token => Some(format!(
            "## `{component}` theme defaults\n\n```dowe\n{component} font:\"manrope\"\n```\n\nDeclares the project-wide default font for `{component}` inside `design`. A `font` prop on one component instance always wins. The configured family is included in generated font assets.\n\n**Accepted props**\n\n- `font`: one quoted Dowe font token"
        )),
        ("fonts", "fonts", _) => Some(
            "## `fonts`\n\n```dowe\nfonts default:\"inter\" install:[\"inter\"]\n```\n\nConfigures project font families from Dowe's built-in catalog. The default family is included in generated targets even when it is absent from `install`.\n\n**Accepted props**\n\n- `default`: one quoted font token; defaults to `\"inter\"`\n- `install`: ordered array of additional quoted font tokens\n\n**Font tokens**\n\n`\"system\"`, `\"inter\"`, `\"roboto\"`, `\"montserrat\"`, `\"lato\"`, `\"poppins\"`, `\"manrope\"`, `\"quicksand\"`, `\"lora\"`, `\"syne\"`, `\"jost\"`, `\"puritan\"`"
                .to_string(),
        ),
        ("fonts", "default", _) => Some(
            "### `fonts.default`\n\n**Type:** quoted font token\n\nSelects the project-wide default font. Dowe uses `\"inter\"` when this prop is omitted."
                .to_string(),
        ),
        ("fonts", "install", _) => Some(
            "### `fonts.install`\n\n**Type:** array of quoted font tokens\n\nAdds font families to the effective generated font set even when no View uses them directly. Values must be unique."
                .to_string(),
        ),
        _ => None,
    }
}

pub(super) fn server_documentation(name: &str) -> Option<String> {
    let entry = SERVER_DOCUMENTATION
        .iter()
        .find(|entry| entry.name == name)?;
    let mut output = format!(
        "## `{}`\n\n```dowe\n{}\n```\n\n{}",
        entry.name, entry.signature, entry.description
    );
    if entry.name == "main" {
        output.push_str(
            "\n\n**Accepted props**\n\n- None\n\n**Accepted children**\n\n- `app` (`name` and `bundle` metadata)\n- `views:<symbol|array>` (one or more imported view route graphs)\n- `server` (optional server target)\n- `desktop` (optional desktop server container)",
        );
    }
    Some(output)
}

pub(super) fn server_prop_documentation(line: &str, prop: &str) -> Option<String> {
    let marker = format!("{prop}:");
    let entry = SERVER_DOCUMENTATION
        .iter()
        .filter(|entry| line.split_whitespace().any(|token| token == entry.name))
        .find(|entry| entry.signature.contains(&marker))?;
    Some(format!(
        "### `{}.{prop}`\n\n```dowe\n{}\n```\n\nAccepted and validated by the shared Dowe server compiler.",
        entry.name, entry.signature
    ))
}

pub(super) fn server_owner_prop_documentation(owner: &str, prop: &str) -> Option<String> {
    let entry = SERVER_DOCUMENTATION
        .iter()
        .find(|entry| entry.name == owner && server_props(owner).contains(&prop))?;
    Some(format!(
        "### `{owner}.{prop}`\n\n```dowe\n{}\n```\n\nAccepted and validated by the shared Dowe server compiler.",
        entry.signature
    ))
}

pub(super) fn stdlib_documentation(name: &str) -> Option<String> {
    let (namespace, function) = name.split_once('.')?;
    let signature = dowe_stdlib::signature(namespace, function)?;
    Some(format_stdlib(&signature))
}

fn format_stdlib(signature: &StdlibSignature) -> String {
    let mut args = signature
        .required
        .iter()
        .map(|name| format!("{name}:<value>"))
        .collect::<Vec<_>>();
    args.extend(
        signature
            .optional
            .iter()
            .map(|name| format!("[{name}:<value>]")),
    );
    let name = format!("{}.{}", signature.namespace, signature.function);
    format!(
        "## `{name}`\n\n```dowe\n{name} {}\n```\n\n**Returns:** `{}`\n\n{}",
        args.join(" "),
        return_kind(signature.return_kind),
        signature.description
    )
}

fn component_description(name: &str) -> &'static str {
    match name {
        "Box" | "Section" | "Flex" | "Grid" | "Card" | "Scaffold" => {
            "Built-in cross-platform layout component lowered by Dowe for every enabled Views target."
        }
        "Brand" => {
            "Built-in cross-platform identity component for arbitrary logo children with optional navigation."
        }
        "Banner" => {
            "Built-in cross-platform external banner component for arbitrary visual children with required HTTPS navigation."
        }
        "Splash" => {
            "Direct layout or page boundary that replaces normal content while its bound boolean Signal is true."
        }
        "AppBar" | "Footer" | "BottomBar" | "SideNav" | "RailNav" | "Sidebar" | "NavMenu"
        | "Tabs" | "tab" | "Stepper" | "step" | "Drawer" => {
            "Built-in cross-platform navigation and application-shell component."
        }
        "Input" | "Select" | "Option" | "Slider" | "Dropzone" | "ComboBox" | "comboOption"
        | "CsvField" | "csvColumn" | "DragDrop" | "dragGroup" | "dragItem" | "Editor"
        | "ImageCropper" | "Password" | "Phone" | "Pin" | "Textarea" | "Checkbox" | "Color"
        | "Date" | "DateRange" | "RadioGroup" | "Toggle" | "ToggleGroup" => {
            "Built-in cross-platform form and interaction component."
        }
        "Code" | "Video" | "Iframe" | "Device" | "Audio" | "Camera" | "Microphone" | "Image"
        | "Canvas" | "Icon" | "Svg" | "Path" | "Candlestick" | "ArcChart" | "AreaChart"
        | "BarChart" | "LineChart" | "PieChart" | "Table" => {
            "Built-in cross-platform media or data-display component."
        }
        _ => "Built-in Dowe Views component lowered to web, desktop, Android, and iOS targets.",
    }
}

fn component_children(name: &str) -> &'static [(&'static str, &'static str)] {
    match name {
        "Box" | "Section" | "Flex" | "Grid" | "Card" => &[("view components", "(zero or more)")],
        "Brand" => &[("view components", "(one or more identity children)")],
        "Banner" => &[("view components", "(one or more banner children)")],
        "Splash" => &[("view components", "(zero or more splash children)")],
        "Badge" | "Tooltip" | "Marquee" | "Collapsible" => &[("view components", "(one or more)")],
        "Button" | "Title" | "Text" => &[("\"text\"", "(one direct static string)")],
        "Chip" => &[
            ("start", "(optional Svg icon region)"),
            ("\"text\"", "(one direct static string)"),
            ("end", "(optional Svg icon region)"),
        ],
        "Select" => &[
            ("Option", "(one or more option entries)"),
            ("validate", "(zero or more ordered validation rules)"),
        ],
        "Input" | "Date" | "Pin" | "Phone" | "Checkbox" => {
            &[("validate", "(zero or more ordered validation rules)")]
        }
        "ComboBox" => &[("comboOption", "(one or more option entries)")],
        "CsvField" => &[("csvColumn", "(one or more column entries)")],
        "DragDrop" => &[
            ("dragItem", "(direct draggable entry)"),
            ("dragGroup", "(group of draggable entries)"),
        ],
        "dragGroup" => &[("dragItem", "(one or more draggable entries)")],
        "Table" => &[("column", "(one or more column entries)")],
        "Svg" => &[(
            "Path",
            "(one or more static path entries, or none with runtime data)",
        )],
        "AppBar" | "Footer" => &[
            ("top", "(optional full-width region)"),
            ("start", "(optional region)"),
            ("centerX", "(optional region)"),
            ("end", "(optional region)"),
            ("bottom", "(optional full-width region)"),
        ],
        "BottomBar" => &[("tab", "(one or more navigation tabs with one Icon child)")],
        "NavMenu" => &[
            ("item", "(navigation entry)"),
            ("submenu", "(nested navigation entries)"),
            ("megamenu", "(navigation entry with a content region)"),
        ],
        "SideNav" => &[
            ("header", "(optional heading entry)"),
            ("item", "(navigation entry)"),
            ("divider", "(optional separator)"),
            ("submenu", "(nested navigation entries)"),
        ],
        "RailNav" => &[
            ("item", "(icon navigation entry)"),
            ("divider", "(optional separator)"),
        ],
        "Sidebar" => &[
            ("header", "(optional region)"),
            ("body", "(required region)"),
            ("footer", "(optional region)"),
        ],
        "Scaffold" => &[
            ("appBar", "(optional region)"),
            ("start", "(optional region)"),
            ("main", "(required region)"),
            ("end", "(optional region)"),
            ("bottomBar", "(optional region)"),
            ("overlays", "(optional region)"),
        ],
        "Drawer" => &[
            ("header", "(optional region)"),
            ("body", "(required region)"),
            ("footer", "(optional region)"),
            (
                "view components",
                "(also accepted directly as body content)",
            ),
        ],
        "Modal" => &[
            ("header", "(optional region)"),
            ("view components", "(required body content)"),
            ("footer", "(optional region)"),
        ],
        "Avatar" => &[("icon", "(optional region)")],
        "Dropdown" => &[
            ("trigger", "(required region)"),
            ("header", "(optional region)"),
            ("item", "(menu entry)"),
            ("divider", "(optional separator)"),
            ("footer", "(optional region)"),
        ],
        "Command" => &[
            ("item", "(command entry)"),
            ("group", "(group of command entries)"),
        ],
        "AvatarGroup" => &[("item", "(static entry; optional with the items prop)")],
        "TypeWriter" | "ToggleGroup" | "Accordion" | "RadioGroup" => {
            &[("item", "(one or more entries)")]
        }
        "RichText" => &[("mark", "(one or more rich-text runs)")],
        "Map" => &[
            ("marker", "(map marker entry)"),
            ("waypoint", "(route waypoint entry)"),
        ],
        "Carousel" => &[("slide", "(one or more slide entries)")],
        "Fab" => &[("fabAction", "(zero or more secondary actions)")],
        "Tabs" => &[("tab", "(one or more tab entries)")],
        "tab" => &[("view components", "(one or more)")],
        "Stepper" => &[("step", "(one or more ordered step entries)")],
        "step" => &[("view components", "(one or more)")],
        _ => &[],
    }
}

fn prop_type(component: &str, prop: &str) -> String {
    if component == "Icon" && prop == "name" {
        return "quoted catalog icon name or string Signal, constant, or each-item path"
            .to_string();
    }
    if component == "Button" && matches!(prop, "loading" | "disabled") {
        return "boolean Signal or View Store path".to_string();
    }
    if component == "Image" && prop == "src" {
        return "quoted packaged or HTTPS source, or string constant, Signal, or each-item path"
            .to_string();
    }
    if let Some(values) = BuiltinComponent::from_name(component)
        .and_then(|component| component_value_completions(component, prop))
        .filter(|values| !values.is_empty())
    {
        return values
            .iter()
            .map(|value| value.label.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
    }
    match prop {
        "disabled"
        | "checked"
        | "multiple"
        | "autoplay"
        | "defaultOpen"
        | "open"
        | "loading"
        | "sending"
        | "streaming"
        | "hasMore"
        | "bordered"
        | "blurred"
        | "boxed"
        | "floating"
        | "dockOnScroll"
        | "fixed"
        | "hideLabel"
        | "labelFloating"
        | "disableOverlayClose"
        | "hideCloseButton"
        | "hideControls"
        | "hideIndicators"
        | "showNavigation"
        | "showCounter"
        | "disableLoop"
        | "showHex"
        | "showRgb"
        | "showCmyk"
        | "showOklch" => "boolean".to_string(),
        "centerX" | "centerY" => "boolean | responsive boolean".to_string(),
        "template" => "boolean".to_string(),
        "w" | "h" | "minW" | "minH" | "maxW" | "maxH" => {
            "Dowe size, auto or responsive Dowe size".to_string()
        }
        "fillRule" => "quoted nonzero | evenodd".to_string(),
        "rotate" => "number from -180 to 180 | responsive number".to_string(),
        "scale" => "number from 0.5 to 2 | responsive number".to_string(),
        "translateX" | "translateY" => {
            "Dowe scale from -96 to 96 | responsive Dowe scale".to_string()
        }
        "gap" => "Dowe scale value or px value | responsive gap".to_string(),
        "flex" => "\"initial\" | \"auto\" | \"none\" | 1 | responsive flex value".to_string(),
        "p" | "px" | "py" | "pl" | "pr" | "pt" | "pb" | "top" | "right" | "bottom" | "left"
        | "columns" | "rows" | "colSpan" | "rowSpan" | "min" | "max" | "step" | "maxSize"
        | "maxPoints" | "offsetX" | "offsetY" | "autoplayInterval" | "slideWidth"
        | "slideHeight" | "slidesPerView" => "number | responsive number".to_string(),
        name if name.starts_with("on") => "fn reference".to_string(),
        "i18n" | "descriptionI18n" | "statusI18n" => "quoted translation key".to_string(),
        "bind" | "data" | "series" | "items" | "messages" | "scene" | "visible" | "start"
        | "end" => "Signal path".to_string(),
        "show" => {
            "boolean | responsive boolean | boolean Signal path | numeric condition".to_string()
        }
        _ => "Dowe value".to_string(),
    }
}

fn prop_description(prop: &str) -> &'static str {
    match prop {
        name if name.starts_with("on") => {
            "References a visible Dowe fn executed by this component event."
        }
        "bind" => "Creates a two-way binding to a compatible Signal path.",
        "show" => {
            "Controls whether the component participates in layout. Accepts a boolean, responsive booleans, a boolean Signal path, or `{ when:<number Signal> gt|gte|lt|lte:<number> }`."
        }
        "scheme" => "Selects the component's semantic Dowe color family.",
        "variant" => "Selects the component's visual treatment.",
        "boxed" => {
            "Constrains and centers the component's generated content body while preserving its full-width structural container."
        }
        "gap" => {
            "Sets spacing between direct children. Accepts a Dowe scale or px value and responsive gap values; Section defaults to zero."
        }
        "flex" => {
            "Sets this Box, Section, Flex, Grid, or Card item's flex behavior when its direct parent is Section, Box, Flex, or Card. Use initial, auto, none, or 1; Grid children ignore it."
        }
        "centerX" => {
            "Centers Box or Section children horizontally. Accepts a boolean or responsive boolean value and defaults to false."
        }
        "centerY" => {
            "Centers Box or Section children vertically when the container has available height. Accepts a boolean or responsive boolean value and defaults to false."
        }
        "dockOnScroll" => {
            "Animates a fixed floating AppBar into the viewport top edge after the document passes 100px of scroll. Requires `floating:true` and `position:\"fixed\"`."
        }
        "position" => {
            "Controls Box flow and overlay placement with static, relative, absolute, or fixed positioning."
        }
        "align" => {
            "Controls logical text alignment on Text and Title. Use start, center, end, or justify; responsive values keep the same target-neutral meaning."
        }
        "top" | "right" | "bottom" | "left" => {
            "Offsets an absolute or fixed Box using a scalar or responsive Dowe scale value."
        }
        "rotate" => "Rotates the component by a validated number of degrees.",
        "scale" => "Scales the component uniformly around its center.",
        "translateX" | "translateY" => {
            "Moves the component visually on one axis without changing document flow."
        }
        "animation" => "Runs the selected entrance animation when the component appears.",
        "transition" => "Selects the timing preset used by interactive gesture state changes.",
        "gesture" => {
            "Adds portable hover or press feedback while respecting reduced-motion settings."
        }
        "maxW" => "Limits the component width without forcing it to occupy the full limit.",
        "h" => {
            "Sets the component height. Accepts a Dowe scale, full, auto, vh-<scale>, or responsive values."
        }
        "minH" => {
            "Sets the component minimum height. Accepts a Dowe scale, full, auto, vh-<scale>, or responsive values."
        }
        "maxH" => {
            "Limits the component height without adding implicit overflow behavior. Accepts a Dowe scale, full, auto, vh-<scale>, or responsive values."
        }
        "fillRule" => "Selects the portable fill rule used to resolve compound Path regions.",
        "size" => {
            "Selects the component's canonical Dowe size. Text and Title sizes use the shared fluid scale, so a scalar value is responsive by default."
        }
        "startIcon" | "endIcon" => {
            "Selects a quoted Solar icon resolved through the shared Icon catalog and sized from the Chip size."
        }
        "i18n" => {
            "References a translation key for the component's primary visible text while preserving the authored text as fallback."
        }
        "descriptionI18n" => {
            "References a translation key for the secondary description while preserving `description` as fallback."
        }
        "statusI18n" => {
            "References a translation key for the status copy while preserving `status` as fallback."
        }
        _ => "Accepted by this component and validated by the shared Dowe compiler.",
    }
}

fn return_kind(kind: StdlibReturnKind) -> &'static str {
    match kind {
        StdlibReturnKind::Unknown => "unknown",
        StdlibReturnKind::Null => "null",
        StdlibReturnKind::Bool => "boolean",
        StdlibReturnKind::Number => "number",
        StdlibReturnKind::String => "string",
        StdlibReturnKind::Array => "array",
        StdlibReturnKind::Object => "object",
    }
}
