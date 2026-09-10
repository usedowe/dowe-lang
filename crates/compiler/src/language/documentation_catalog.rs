#[rustfmt::skip]
pub(super) const VIEW_COMPONENTS: &[&str] = &[
    "Box", "Section", "Flex", "Grid", "Input", "Select", "Option", "Code", "Video", "Iframe", "Device", "Canvas", "Draw",
    "Candlestick", "Diagram", "ArcChart", "AreaChart", "BarChart", "LineChart", "PieChart", "Table", "Divider",
    "Button", "Brand", "Banner", "ToggleTheme", "SelectTheme", "Fab", "fabAction", "Slider", "Dropzone", "ComboBox",
    "comboOption", "CsvField", "csvColumn", "DragDrop", "dragGroup", "dragItem", "Editor", "ImageCropper",
    "Password", "Phone", "Pin", "Textarea", "Alert", "Icon", "Svg", "Path", "AppBar", "Footer",
    "BottomBar", "NavMenu", "SideNav", "RailNav", "Sidebar", "Scaffold", "Splash", "Drawer", "Avatar", "Badge", "Chip",
    "Skeleton", "Modal", "AlertDialog", "Tooltip", "Toast", "Dropdown", "Command", "AvatarGroup", "ChatBox",
    "Empty", "Marquee", "TypeWriter", "RichText", "Record", "ToggleGroup", "Collapsible", "Countdown", "Map",
    "Audio", "Camera", "Microphone", "Image", "Accordion", "Tree", "Carousel", "Checkbox", "Color", "Date", "DateRange", "RadioGroup", "RadioCard", "Toggle",
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
        name: "ipc",
        signature: "ipc functions:[<imported-function> ...] [databases:<symbol|array>]",
        description: "Registers imported Dowe functions and local Database handles for native View invocation on desktop, Android, or iOS without creating an HTTP listener.",
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
        name: "invoke",
        signature: "invoke <result> fn:<name> [args:{ ... }]",
        description: "Invokes a function registered in a native IPC target; development Web uses the local Dowe IPC bridge and production Web remains HTTP-only.",
    },
    ServerDocumentation {
        name: "notify",
        signature: "notify <result> user:<value> title:<value> body:<value> [id:<value>] [category:<value>] [route:<value>] [data:<object>] [tag:<value>]",
        description: "Persists a validated notification intent for the authenticated user's active installations. Use process or chat categories and internal routes only; provider delivery is durable and observable.",
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


