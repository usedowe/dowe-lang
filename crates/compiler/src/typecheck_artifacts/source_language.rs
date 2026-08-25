use crate::model::GeneratedFile;
use std::path::PathBuf;

pub fn typecheck_artifacts() -> Vec<GeneratedFile> {
    vec![
        generated(
            "language/source-format.json",
            SOURCE_FORMAT,
            "LanguageSupport",
            "source",
        ),
        generated(
            "language/server-surface.json",
            SERVER_SURFACE,
            "LanguageSupport",
            "server",
        ),
        generated(
            "language/views-surface.json",
            VIEWS_SURFACE,
            "LanguageSupport",
            "views",
        ),
        generated(
            "language/config-surface.json",
            CONFIG_SURFACE,
            "LanguageSupport",
            "config",
        ),
        generated(
            "language/i18n-surface.json",
            I18N_SURFACE,
            "LanguageSupport",
            "i18n",
        ),
    ]
}

pub fn obsolete_typecheck_artifacts() -> Vec<PathBuf> {
    vec![
        PathBuf::from("tsconfig.json"),
        PathBuf::from("server-tsconfig.json"),
        PathBuf::from("views-tsconfig.json"),
        PathBuf::from("types/server.d.ts"),
        PathBuf::from("types/views.d.ts"),
    ]
}

fn generated(
    relative_path: impl Into<PathBuf>,
    content: &str,
    kind: &str,
    target: &str,
) -> GeneratedFile {
    GeneratedFile {
        relative_path: relative_path.into(),
        content: content.to_string(),
        kind: kind.to_string(),
        target: target.to_string(),
    }
}

const SOURCE_FORMAT: &str = r#"{
  "format": "dowe-source-format",
  "extension": ".dowe",
  "indentation": {
    "spacesPerLevel": 2,
    "tabs": "rejected"
  },
  "moduleRoot": ".",
  "unsupportedAuthoringExtensions": [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"],
  "declaredTypes": {
    "declaration": "type User",
    "field": "name:string",
    "optionalField": "id?:string",
    "arrays": "User[]",
    "scope": "local file or imported from types"
  },
  "sharedTypes": ["any imported pure type .dowe module", "classified by type declarations", "signal users type:User[] value:[]", "const body:User value:req.json"],
  "imports": {
    "forms": ["./", "../", "@/"],
    "projectRootAlias": "@/",
    "extension": ".dowe",
    "packages": "rejected",
    "urls": "rejected",
    "outsideSourceRoot": "rejected",
    "assetsRoot": "assets",
    "assetsImportable": false
  },
  "propStrings": "quoted static literals; bare values are validated references only",
  "textChildren": "Text, Title, and Button visible copy uses one direct double-quoted string child; dynamic text uses exactly one braced binding such as {blog.title}; every other string is literal"
}
"#;

const SERVER_SURFACE: &str = r#"{
  "root": "main.dowe",
  "blocks": ["type", "entity", "seeder", "database", "cache", "vector", "queue", "main", "app", "views", "server", "tls", "desktop", "endpoints", "group", "get", "post", "put", "delete", "patch", "route", "websocket", "udp", "tcp", "packet", "connection", "rtp", "model", "init", "handler", "middleware", "fn"],
  "httpMethods": ["get", "post", "put", "delete", "patch"],
  "actions": ["functionName result args:{ ... }", "const binding value:req.json", "namespace result source:\"stdlibFunction\"", "request result source:\"query|rawQuery|header|cookie|bytes\"", "file result source:\"write|read|exists|delete\" root:<path> path:<path>", "password result source:\"hash|verify\" value:<password>", "queue appQueue provider:\"dowe|rabbitmq|cloudflare|vercel\" host:<value> port:<value> account:<value> secret:<value> vhost:<value>", "msg sent conn:appQueue.publish queue:<value> payload:<json>", "ws result source:\"json\"", "agent result source:\"chat\" request:request", "session result cache:cache database:database token:token", "return status:201 json:value", "return reverse:route.upstreams strategy:\"roundRobin\" state:route.state loadingUrl:route.loadingUrl errorUrl:route.errorUrl", "return value", "next", "response text", "response json", "send ws", "bridge sse", "task fn:functionName args:{ ... }", "task fn:functionName args:{ event:{ ... } } after:\"headers\"", "task args:{ ... } <server statements...>", "cron fn:functionName schedule:\"0 * * * *\" args:{ ... }", "log", "info", "warn", "error"],
  "request": ["req.params", "req.json", "req.context", "request result source:\"query\"", "request result source:\"rawQuery\"", "request result source:\"header\" name:\"Range\"", "request result source:\"cookie\" name:\"session\"", "request result source:\"bytes\"", "ws result source:\"json\""],
  "outboundHttp": ["http <binding>", "method:\"get\"", "base", "path", "bearer", "headers", "json", "mode:\"json\"", "mode:\"proxy\"", "mode:\"bytes\"", "redirect:\"follow\"", "redirect:\"manual\"", "redirect:\"error\"", "maxRedirects", "timeoutMs"],
  "serverProcess": ["spawn <binding>", "command", "args", "cwd", "timeoutMs", "maxOutputBytes", "background"],
  "serverCrypto": ["crypto <binding>", "encryption:\"aesCtr\"", "encryption:\"cencAesCtr\"", "data", "key", "iv", "subsamples"],
  "standardLibrary": ["str result source:\"trim\"", "str result source:\"join\"", "math result source:\"sum\"", "parse result source:\"int\"", "parse result source:\"float\"", "parse result source:\"json\"", "parse result source:\"svg\"", "url result source:\"parse\"", "url result source:\"querySet\"", "csv result source:\"parse\"", "sort result source:\"by\"", "list result source:\"filterContains\"", "json result source:\"get\"", "json result source:\"stringify\"", "date result source:\"now\"", "id result source:\"ulid\""],
  "agent": ["agent result source:\"chat\" request:request", "return agent:upstream request:request"],
  "middleware": ["group middleware:[name]", "get path:\"/...\" middleware:[name]", "websocket path:\"/...\" middleware:[name]", "middleware name params:{ ... }", "implicit req and next", "bearer token value:req.header.Authorization", "session verified cache:appCache database:appDb token:token maxAge:2592000", "jwt verified secret:env.JWT_SECRET algorithm:\"HS256\" token:req.query.token"],
  "serverFunctions": ["fn name params:{ input:Input } return:\"Output\"", "functionName result args:{ input:value }", "return value:{ ... }"],
  "backgroundJobs": ["task", "named task", "inline task", "cron", "isolated process", "fire-and-forget", "immediate source order by default", "after:\"headers\" only on a direct reverse-proxy handler", "after headers requires args.event object and runs once after real upstream headers", "handler and function bindings serialized as JSON", "static init and cron args", "inline body uses args and imported configuration only"],
  "reverseProxy": ["return reverse:route.url", "return reverse:route.upstreams strategy:\"roundRobin\" state:route.state loadingUrl:route.loadingUrl errorUrl:route.errorUrl", "required Cache.get source", "filter disabled or non-ready runtimes", "temporary loading and error redirects", "preserve method path query body status headers and streaming", "after headers telemetry overwrites event.status method path latencyMs bytesIn bytesOut only after real upstream headers", "reject request-controlled upstreams"],
  "serverConfigModules": ["any imported .dowe module", "classified by database, cache, vector, or queue declaration", "database appDb provider:\"dowe\" host:env.DB_HOST port:env.DB_PORT account:env.DB_USER secret:env.DB_PASSWORD name:\"app\" entities:[Users] seeders:[Bootstrap]", "cache appCache provider:\"kv|redis|dowe\" or provider:env.CACHE_PROVIDER host:env.CACHE_HOST port:env.CACHE_PORT account:env.CACHE_USER secret:env.CACHE_PASSWORD name:env.CACHE_NAME", "vector appVector provider:\"dowe\" host:env.VECTOR_HOST port:env.VECTOR_PORT account:env.VECTOR_USER secret:env.VECTOR_PASSWORD name:env.VECTOR_DATABASE", "queue appQueue provider:\"dowe|rabbitmq|cloudflare|vercel\" host:env.QUEUE_HOST port:env.QUEUE_PORT account:env.QUEUE_USER secret:env.QUEUE_PASSWORD vhost:env.QUEUE_VHOST"],
  "tls": ["tls mode:\"acme\" domains:[\"example.com\"] email:\"admin@example.com\" staging:false httpPort:80", "tls mode:\"local\" domains:[\"localhost\"]", "domainsFrom KV, Database, or authenticated HTTPS endpoint", "automatic ACME renewal", "server only"],
  "jwt": {
    "jws": ["HS256"],
    "jwe": ["dir", "A256GCM"],
    "secrets": "server env variables only"
  },
  "inferredReferences": ["store.insert field", "store.insert id", "store.update changed", "store.delete changed", "kv.set ok", "kv.set key", "kv.delete deleted", "kv.clear cleared", "emb.upsert id", "emb.upsert dimensions", "emb.upsert created", "emb.delete deleted", "msg.publish ok", "msg.publish id"],
  "declaredTypes": ["const body:Type value:req.json", "validated request JSON", "typed body references"],
  "resolvedLogValues": true,
  "handlers": ["get path:\"/...\" handler:name", "post path:\"/...\" handler:name", "put path:\"/...\" handler:name", "patch path:\"/...\" handler:name", "delete path:\"/...\" handler:name"],
  "websocketHandlers": ["websocket path:\"/...\" middleware:[name]", "open", "message", "close", "drain", "message ws", "send ws json:{ ... }", "bridge sse:upstream to:ws"],
  "protocolTransports": ["udp name:\"sip-udp\" bind:\"0.0.0.0\" port:5060", "tcp name:\"sip-tcp\" bind:\"0.0.0.0\" port:5060", "packet pkt", "connection conn", "pkt.text", "pkt.bytes", "pkt.addr", "rtp bind:\"0.0.0.0\" min:40000 max:40100"],
  "localModels": ["model name:\"voice-vad\" kind:\"vad.silero\" engine:\"candle\" format:\"onnx\" source:\"assets/silero_vad.onnx\" sampleRates:[8000 16000]", "source under assets/", "web target: unavailable"],
  "hostRuntime": "rust",
  "nodeRuntime": false
}
"#;

const VIEWS_SURFACE: &str = r#"{
  "root": "main.dowe",
  "exports": ["views", "layout", "page", "type", "store"],
  "startup": ["one direct init per layout or page", "init uses ordered view function statements", "redirect path:\"/...\" replaces the active internal route", "one direct Splash bind:booleanSignal", "Splash true shows its children", "Splash false shows normal roots", "layout keeps one normal root", "page keeps one or more normal roots", "web Android iOS"],
  "metadata": {"declaration": "meta name:\"title\" content:\"Page title\"", "placement": "direct layout or page child", "inheritance": "active layout then page by name", "names": ["title", "description", "keywords", "robots", "canonical", "og:title", "og:description", "og:image", "og:image:alt", "og:type", "og:url", "og:site_name", "twitter:card", "twitter:title", "twitter:description", "twitter:image", "twitter:image:alt", "twitter:site", "twitter:creator"], "target": "web SSR and browser routing only"},
  "components": ["Box", "Section", "Flex", "Grid", "Input", "Select", "Option", "Slider", "Dropzone", "ComboBox", "comboOption", "CsvField", "csvColumn", "DragDrop", "dragGroup", "dragItem", "Editor", "ImageCropper", "Password", "Phone", "Pin", "Textarea", "Code", "Video", "Device", "Canvas", "Candlestick", "ArcChart", "AreaChart", "BarChart", "LineChart", "PieChart", "Table", "Pagination", "Divider", "Button", "ToggleTheme", "SelectTheme", "Fab", "fabAction", "Alert", "Icon", "Svg", "Path", "Card", "AppBar", "Footer", "BottomBar", "SideNav", "RailNav", "Sidebar", "NavMenu", "Scaffold", "Splash", "Tabs", "tab", "Stepper", "step", "Drawer", "Avatar", "AvatarGroup", "Badge", "Chip", "Skeleton", "Modal", "AlertDialog", "Tooltip", "Toast", "Dropdown", "Command", "ChatBox", "Empty", "Marquee", "TypeWriter", "RichText", "mark", "Record", "ToggleGroup", "Collapsible", "Countdown", "Map", "marker", "waypoint", "Audio", "Camera", "Microphone", "Image", "Accordion", "Carousel", "Checkbox", "Color", "Date", "DateRange", "RadioGroup", "Toggle", "Title", "Text"],
  "slots": ["children", "top", "start", "center", "end", "bottom", "appBar", "main", "bottomBar", "overlays", "content", "header", "body", "footer", "trigger", "icon", "group", "item", "mark", "marker", "waypoint", "divider", "comboOption", "csvColumn", "dragGroup", "dragItem", "tab"],
  "routing": ["main views:viewRoutes", "main views:[dashboardRoutes docsRoutes]", "server endpoints:apiRoutes", "server endpoints:[userRoutes blogRoutes]", "endpoints apiRoutes", "endpoint groups are one level", "group path:\"/api\" middleware:[requireBearer]", "get path:\"/status\" handler:status", "websocket path:\"/events\" middleware:[requireSocketToken]", "views viewRoutes", "view groups contain direct routes only", "group path:\"/\" layout:Layout platform:\"web\"", "group path:\"/\" layout:Layout platform:[\"desktop\" \"ios\" \"android\"]", "route path:\"\" page:Page platform:\"desktop\"", "platform values: web desktop android ios"],
  "flexDirection": ["direction:\"row\"", "direction:\"column\"", "direction:{ xs:\"column\" md:\"row\" }", "wrap:true"],
  "flexItem": ["flex:\"initial\"", "flex:\"auto\"", "flex:\"none\"", "flex:1", "flex:{ xs:1 md:\"none\" }", "Section Box Flex Grid Card", "only effective under Section Box Flex Card"],
  "sizing": ["w", "h", "minW", "minH", "maxW", "maxH", "w minW maxW accept full or container sm through 7xl", "responsive values", "h minH maxH accept auto or vh-<scale>"],
  "boxPositioning": ["Box position:\"relative\"", "Box position:\"absolute\" top:4 right:{ xs:4 md:6 }", "Box position:\"fixed\" bottom:4 right:4", "absolute requires direct relative Box parent", "fixed is rooted outside route scrolling"],
  "sectionCenter": ["center:true", "center:false", "center:{ xs:false md:true }", "omitted center uses false", "web Android iOS"],
  "boxCenter": ["center:true", "center:false", "center:{ xs:false md:true }", "omitted center uses false", "web Android iOS"],
  "sectionGap": ["gap:3", "gap:\"8px\"", "gap:{ xs:2 md:4 }", "omitted gap uses 0", "web Android iOS"],
  "viewStores": ["any imported .dowe module", "classified by store declaration", "store session persistent:true value:{ ... }", "persistent targets: localStorage SharedPreferences UserDefaults"],
  "viewConstants": ["const plans value:[...]", "immutable", "web Android iOS"],
  "chipIconProps": {"props": ["startIcon", "endIcon"], "values": "quoted static Solar icon names", "sizeMapping": {"xs": 12, "sm": 14, "md": 16, "lg": 20, "xl": 24}, "regions": ["start", "end"]},
  "reactivity": ["signal", "signal session scope:\"global\" storage:\"local\" value:{ ... }", "fn params:{ form:Form } return:\"boolean\"", "request route:\"/api/...\"", "request path:\"/...\"", "request base:env.NAME", "request headers:{ Authorization:session.authorization }", "onSuccess alert:\"...\"", "onError alert:\"...\"", "implicit BACKEND_URL for /api", "set target value:signal.field", "set target value:!signalBool", "set target value:true", "reset", "Input bind:signal.field", "Select bind:signal.field", "ComboBox bind:signal.field", "Editor bind:signal.field", "ImageCropper bind:signal.field", "Password bind:signal.field", "Phone bind:signal.field", "Pin bind:signal.field", "Textarea bind:signal.field", "Slider bind:signal.number", "Button onClick:fn", "\"Fab onClick:fn\"", "fabAction onClick:fn", "Avatar onClick:fn", "AvatarGroup items:avatars item onClick:fn", "ChatBox messages:messages onSend:sendMessage", "ChatBox loading:isLoading sending:isSending streaming:isStreaming hasMore:hasMore", "Record onStart:onRecordStart onPause:onRecordPause onConfirm:onRecordConfirm", "ToggleGroup value:signalString onChange:fn", "Pagination bind:page total:50 pageSize:10 onChange:loadPage", "Countdown onComplete:fn", "Map onLocation:fn onLocationError:fn onRoute:fn marker onClick:fn", "Empty onClick:fn", "Chip onClose:fn", "Modal onClose:fn open:signalBool", "AlertDialog onConfirm:fn onCancel:fn open:signalBool", "Dropdown item onClick:fn", "Command open:signalBool item onClick:fn", "Toast source:signalObject", "show:signalBool", "Drawer open:signalBool", "\"{blog.title}\" dynamic text child", "\"blog.title\" literal text child", "\"RichText mark text:\"...\" style:\"grad\"\"", "Code content:\"\"\"...\"\"\"", "Video src:\"https://...\"", "Candlestick data:signal stream:\"/api/...\"", "LineChart data:signal series:signal", "AreaChart data:signal series:signal", "BarChart data:signal", "ArcChart data:signal", "PieChart data:signal", "Table data:signal column field:\"name\" label:\"Name\"", "Tabs tab children", "Stepper step children", "Fab fabAction children", "ComboBox comboOption children", "CsvField csvColumn children", "DragDrop dragItem dragGroup children", "NavMenu item submenu megamenu children", "Scaffold appBar start main end bottomBar overlays regions", "Modal header footer regions", "Drawer header body footer regions", "Dropdown trigger item divider regions", "Command group item entries", "Marquee children", "TypeWriter item text:\"...\"", "Collapsible label:\"Details\" children", "Map marker id:\"office\" lat:4.71 lng:-74.07", "Divider orientation:\"horizontal\"", "Svg Path children", "AppBar mobileMenu mobileMenuOpen:<signal> header body footer plus top start center end bottom regions", "BottomBar tab href label Icon featured"],
  "standardLibrary": ["str.trim", "str.lower", "str.upper", "str.length", "math.sum", "parse.int", "parse.float", "parse.json", "parse.svg", "url.parse", "url.querySet", "csv.parse", "sort.by", "list.filterContains", "json.get", "json.stringify", "date.now"],
  "inlineClick": ["onClick:fn", "onClick:{ set:openDrawer value:!openDrawer }", "onClick:{ set:counter add:1 }", "onClick:{ set:message append:\"!\" }", "set targets: Signals and View Store paths"],
  "reactiveProps": {"Icon": {"stringSignals": ["name"], "catalog": "shared icon catalog with initial fallback"}, "Button": {"stringSignals": ["variant", "scheme", "size", "rounded"], "booleanSignals": ["loading"], "conditionalIcons": {"props": ["iconStart", "iconEnd"], "boolean": "when:path", "numeric": ["gt", "gte", "lt", "lte"]}}, "SideNav": {"stringSignals": ["variant", "scheme", "size"], "booleanSignals": ["wide"]}, "Image": {"stringSignals": ["src"], "accepted": "quoted packaged or HTTPS source, or a string constant, Signal, or each-item path"}, "Code": {"template": "validated {signal.path} placeholders in static content"}},
  "signalPathValidation": "known object fields and supported target scalar type are checked before generation",
  "declaredTypes": ["signal form type:Form value:{ ... }", "signal rows type:Row[] value:[]"],
  "localizedLabels": ["Button i18n:\"actions.save\"", "NavMenu item label:\"Views\" i18n:\"navigation.views\" description:\"Catalog\" descriptionI18n:\"navigation.catalog\"", "SideNav item label:\"Views\" i18n:\"navigation.views\" status:\"Ready\" statusI18n:\"navigation.ready\"", "tab id:\"overview\" label:\"Overview\" i18n:\"tabs.overview\"", "BottomBar tab href:\"/home\" label:\"Home\" featured:true Icon child"],
  "staticStrings": ["Text i18n:\"home.hero.summary\"", "Title i18n:\"home.hero.title\"", "RichText i18n:\"home.hero.summary\"", "RichText mark text:\"Launch\" style:\"grad\" scheme:\"primary\"", "Input label:\"...\"", "Select placeholder:\"...\"", "Option value:\"...\"", "ComboBox placeholder:\"Choose\" searchPlaceholder:\"Search\"", "comboOption value:\"admin\" label:\"Administrator\"", "CsvField buttonText:\"Upload CSV\" modalTitle:\"Review import\"", "csvColumn name:\"email\" label:\"Email\"", "DragDrop direction:\"horizontal\" emptyText:\"No items\"", "dragGroup id:\"todo\" title:\"Todo\"", "dragItem id:\"draft\" label:\"Draft\"", "Editor placeholder:\"Write notes\"", "ImageCropper shape:\"circle\"", "Password weakLabel:\"Weak\" mediumLabel:\"Medium\" strongLabel:\"Strong\"", "Phone country:\"US\"", "Pin type:\"number\"", "Textarea rows:4 maxLength:160", "RadioGroup orientation:\"horizontal\"", "Slider label:\"Volume\" min:0 max:100 step:5", "Dropzone accept:\"image/*\" placeholder:\"Drop files\"", "ToggleTheme lightLabel:\"Light mode\" darkLabel:\"Dark mode\"", "Fab position:\"bottom-right\" icon:\"plus\"", "fabAction label:\"Docs\" icon:\"link\" href:\"/docs\"", "Record name:\"voice\" maxDuration:90 variant:\"solid\"", "ToggleGroup selected:\"map\" size:\"sm\" ariaLabel:\"Display mode\"", "ToggleGroup item id:\"map\" label:\"Map\"", "Collapsible label:\"Details\" defaultOpen:true", "Countdown target:\"2030-01-01T00:00:00Z\" size:\"md\"", "Map centerLat:4.7109 centerLng:-74.0721 zoom:12 height:\"360px\"", "Map marker id:\"office\" label:\"Office\" icon:\"start\"", "Code language:\"dowe\"", "Video aspect:\"horizontal\"", "Candlestick emptyLabel:\"No candle data\"", "LineChart curve:\"smooth\" palette:\"ocean\"", "AreaChart fillOpacity:0.3 legendPosition:\"bottom\"", "BarChart grouped:true size:\"lg\"", "ArcChart startAngle:-90 endAngle:270", "PieChart donut:true donutWidth:72", "Table scheme:\"surface\"", "Table emptyTitle:\"No data\"", "column field:\"name\" label:\"Name\" align:\"start\"", "Tabs variant:\"line\"", "Tabs scheme:\"primary\"", "Tabs position:\"top\"", "tab id:\"overview\" label:\"Overview\"", "NavMenu scheme:\"muted\"", "NavMenu item label:\"Docs\" href:\"/docs\"", "Sidebar body SideNav", "Scaffold boxed:true", "Section boxed:true", "Box h:\"vh-16\"", "Divider scheme:\"muted\"", "Path fill:\"none\" fillRule:\"evenodd\"", "AppBar scheme:\"surface\"", "Footer scheme:\"background\"", "BottomBar variant:\"solid\"", "Drawer position:\"start\"", "Avatar name:\"Ada\" size:\"md\" status:\"online\"", "AvatarGroup size:\"md\" max:4 inline:false", "AvatarGroup item src:\"/team/ada.png\" name:\"Ada\"", "ChatBox mode:\"conversation\" placeholder:\"Type a message...\"", "Empty type:\"data\" title:\"No data\" actionLabel:\"View more\"", "Marquee speed:\"normal\" orientation:\"horizontal\" fade:true", "TypeWriter typeSpeed:100 deleteSpeed:50 item text:\"Build systems\"", "Badge text:\"3\" position:\"top-right\"", "Chip variant:\"solid\" size:\"sm\" rotate:-7 transition:\"spring\" gesture:\"lift\"", "Skeleton variant:\"rounded\" animation:\"wave\"", "Modal scheme:\"surface\"", "AlertDialog title:\"Are you sure?\"", "Tooltip label:\"More\" position:\"top\"", "Toast type:\"success\" position:\"bottom-left\"", "Dropdown item label:\"Docs\" href:\"/docs\"", "Command group label:\"Navigation\"", "Box animation:\"fadeIn\"", "Section background:\"aurora\"", "Section background values: aurora sunrise ocean meadow slate", "Card animation:\"slideUp\"", "View transforms: rotate scale translateX translateY", "View transitions: none quick smooth spring", "View gestures: none lift press grow tilt"],
  "visibility": ["show:true", "show:false", "show:{ xs:false md:true }", "show:signalBool", "show:{ when:signalNumber gt:10 }", "numeric show comparators: gt gte lt lte"],
  "controlFlow": {
    "if": "static true and false conditions are supported until data surfaces are specified",
    "each": "renders signal arrays with deterministic key paths"
  },
  "navigation": ["href", "navigate", "history", "target", "externalMode"],
  "serverApisAvailable": false
}
"#;

const I18N_SURFACE: &str = r#"{
  "root": "i18n/<locale>.dowe",
  "locale": "two or three lowercase language letters",
  "rootBlock": "translations",
  "entry": "home -> hero -> title \"...\"",
  "legacyEntry": "translation key:\"home.hero.title\" value:\"...\"",
  "defaultLocale": "exactly one translations default:true catalog",
  "components": ["Text", "Title", "RichText"],
  "web": "deterministic locale chunks with navigator.languages detection",
  "desktop": "reuses web locale chunks",
  "android": "native res/values resources",
  "ios": "native Localizable.strings resources"
}
"#;

const CONFIG_SURFACE: &str = r##"{
  "themeRoot": "theme.dowe",
  "themeRootBlock": "theme",
  "envRoot": ".env",
  "envExampleRoot": ".env.example",
  "serverRoot": "main.dowe",
  "serverConfigModules": "any imported module classified by database, cache, or vector declarations",
  "blocks": ["theme", "fonts", "design", "theme", "colors", "radius", "shadow", "shadowColor", "border", "borderColor", "scheme", "variant", "main", "app", "views", "server", "tls", "endpoints", "cors", "udp", "tcp", "rtp", "model"],
  "app": {
    "declaration": "main app name:\"Dowe Dev\" bundle:\"dev.dowe.generated\"",
    "bundle": "reverse-dns"
  },
  "fonts": ["system", "inter", "roboto", "montserrat", "lato", "poppins", "manrope", "quicksand", "lora", "syne", "jost", "puritan"],
  "themes": {
    "defaultTheme": "light",
    "builtInInheritance": ["light", "dark"],
    "runtimeSwitching": false,
    "colors": {
      "declaration": "colors: -> primary color:\"#2563eb\" text:\"#ffffff\" title:\"#ffffff\"",
      "families": ["primary", "secondary", "accent", "muted", "background", "surface", "success", "info", "warning", "danger", ],
      "roles": ["color", "text", "title"],
      "flatRoleAuthoring": "rejected"
    }
  },
  "colorValues": ["#RGB", "#RRGGBB", "#RRGGBBAA"],
  "themeRadius": "radius:<non-negative integer>",
  "componentDefaults": {
    "slots": ["card", "button", "chip", "avatar", "tabs", "ui", "text", "title"],
    "fontSlots": ["text", "title"],
    "radius": ["xs", "sm", "md", "lg", "xl", "full"],
    "shadow": ["xs", "sm", "md", "lg", "xl"],
    "border": [1, 2, 3, 4],
    "scheme": ["primary", "secondary", "accent", "muted", "background", "surface", "success", "info", "warning", "danger"],
    "variant": ["solid", "outline", "outlined", "ghost", "line"],
    "tabsVariant": ["solid", "outlined", "line", "ghost", "pills"]
  },
  "environment": {
    "declaration": "BACKEND_URL=",
    "references": "env.NAME",
    "resolutionOrder": ["operating-system", ".env"],
    "exampleValuesAreEffective": false,
    "clientExposure": "inferred from view references",
    "clientValuesArePublic": true
  },
  "cors": {
    "declaration": "cors target:\"server\" devOrigins:true origins:[\"http://127.0.0.1:56035\"] methods:[\"GET\" \"POST\" \"PATCH\" \"DELETE\"] headers:[\"Content-Type\"] credentials:false maxAge:600",
    "targets": ["server", "desktop", "all"],
    "origins": ["exact-http-origin", "exact-https-origin", "*"],
    "devOrigins": "dowe-dev-managed-client-origins",
    "credentialsWithWildcard": false
  },
  "obsoleteConfig": ["dowe.json", "env.dowe", "src/config.dowe", "src/main.dowe", "src/theme.dowe", "src/env.dowe"]
}
"##;
