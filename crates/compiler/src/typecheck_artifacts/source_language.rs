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
  "sourceRoot": "src",
  "unsupportedAuthoringExtensions": [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"],
  "declaredTypes": {
    "declaration": "type User",
    "field": "name:string",
    "optionalField": "id?:string",
    "arrays": "User[]",
    "scope": "local file"
  },
  "imports": {
    "relativeOnly": true,
    "extension": ".dowe",
    "packages": "rejected",
    "urls": "rejected",
    "outsideSourceRoot": "rejected"
  },
  "propStrings": "quoted static literals; bare values are validated references only",
  "textChildren": "Text, Title, and Button visible copy uses one direct double-quoted string child"
}
"#;

const SERVER_SURFACE: &str = r#"{
  "root": "src/main.dowe",
  "blocks": ["type", "main", "server", "desktop", "route", "websocket", "init", "handler", "middleware"],
  "httpMethods": ["GET", "POST", "PUT", "DELETE", "PATCH"],
  "actions": ["let", "return", "return continue", "response text", "response json", "response proxy", "response agent", "send ws", "bridge sse", "log", "info", "warn", "error"],
  "request": ["req.params", "req.json()", "req.header", "req.context", "ws.json"],
  "outboundHttp": ["http.request", "http.get", "http.post", "method:\"GET\"", "base", "path", "bearer", "headers", "json", "mode:\"json\"", "mode:\"proxy\"", "redirect:\"follow\"", "redirect:\"manual\"", "redirect:\"error\"", "maxRedirects", "timeoutMs"],
  "standardLibrary": ["str.trim", "str.lower", "str.upper", "str.length", "math.sum", "parse.int", "parse.float", "parse.json", "url.parse", "url.querySet", "csv.parse", "sort.by", "list.filterContains", "json.get", "json.stringify", "date.now"],
  "agent": ["agent.chat", "return response agent:upstream request:request"],
  "middleware": ["route middleware:[name]", "middleware name async req", "bearer", "jwt.verify", "jwt.decrypt", "jwt.sign", "jwt.encrypt"],
  "jwt": {
    "jws": ["HS256"],
    "jwe": ["dir", "A256GCM"],
    "secrets": "server env variables only"
  },
  "inferredReferences": ["store.insert field", "store.insert id", "store.update changed", "store.delete changed", "kv.set ok", "kv.set key", "kv.delete deleted", "kv.clear cleared"],
  "declaredTypes": ["let body:Type = await req.json()", "validated request JSON", "typed body references"],
  "resolvedLogValues": true,
  "handlers": ["method GET handler:name", "method POST handler:name", "method PATCH handler:name", "method DELETE handler:name"],
  "websocketHandlers": ["open", "message", "close", "drain", "message ws", "send ws json:{ ... }", "bridge sse:upstream to:ws"],
  "hostRuntime": "rust",
  "nodeRuntime": false
}
"#;

const VIEWS_SURFACE: &str = r#"{
  "root": "src/views.dowe",
  "exports": ["views", "layout", "page", "type"],
  "components": ["Box", "Section", "Flex", "Grid", "Input", "Select", "Option", "Slider", "Dropzone", "ComboBox", "comboOption", "CsvField", "csvColumn", "DragDrop", "dragGroup", "dragItem", "Editor", "ImageCropper", "PasswordField", "PhoneField", "PinField", "Textarea", "Code", "Video", "Candlestick", "ArcChart", "AreaChart", "BarChart", "LineChart", "PieChart", "Table", "Divider", "Button", "ToggleTheme", "Fab", "fabAction", "Alert", "Svg", "Path", "Card", "AppBar", "Footer", "BottomBar", "SideNav", "Sidebar", "NavMenu", "Scaffold", "Tabs", "tab", "Drawer", "Avatar", "AvatarGroup", "Badge", "Chip", "Skeleton", "Modal", "AlertDialog", "Tooltip", "Toast", "Dropdown", "Command", "ChatBox", "Empty", "Marquee", "TypeWriter", "RichText", "mark", "Record", "ToggleGroup", "Collapsible", "Countdown", "Map", "marker", "waypoint", "Audio", "Image", "Accordion", "Carousel", "Checkbox", "Color", "Date", "DateRange", "RadioGroup", "Toggle", "Title", "Text"],
  "slots": ["children", "start", "center", "end", "appBar", "main", "bottomBar", "content", "header", "body", "footer", "trigger", "icon", "group", "item", "mark", "marker", "waypoint", "divider", "comboOption", "csvColumn", "dragGroup", "dragItem"],
  "routing": ["route path:\"/\" layout:Layout platform:\"web\"", "route path:\"/\" layout:Layout platform:[\"desktop\",\"ios\",\"android\"]", "page path:\"\" component:Page platform:\"desktop\"", "platform values: web desktop android ios"],
  "reactivity": ["signal", "action", "request route:\"/api/...\"", "request path:\"/...\"", "request base:env.NAME", "onSuccess alert:\"...\"", "onError alert:\"...\"", "implicit BACKEND_URL for /api", "assign", "assign target source:str.trim value:signal.field", "reset", "Input bind:signal.field", "Select bind:signal.field", "ComboBox bind:signal.field", "Editor bind:signal.field", "ImageCropper bind:signal.field", "PasswordField bind:signal.field", "PhoneField bind:signal.field", "PinField bind:signal.field", "Textarea bind:signal.field", "Slider bind:signal.number", "Button onClick:action", "\"Fab onClick:action\"", "fabAction onClick:action", "Avatar onClick:action", "AvatarGroup items:avatars item onClick:action", "ChatBox messages:messages onSend:sendMessage", "ChatBox loading:isLoading sending:isSending streaming:isStreaming hasMore:hasMore", "Record onStart:onRecordStart onPause:onRecordPause onConfirm:onRecordConfirm", "ToggleGroup value:signalString onChange:action", "Countdown onComplete:action", "Map onLocation:action onLocationError:action onRoute:action marker onClick:action", "Empty onClick:action", "Chip onClose:action", "Modal onClose:action open:signalBool", "AlertDialog onConfirm:action onCancel:action open:signalBool", "Dropdown item onClick:action", "Command open:signalBool item onClick:action", "Toast source:signalObject", "show:signalBool", "Drawer open:signalBool", "\"Button text child\"", "\"Title text child\"", "\"Text text child\"", "\"RichText mark text:\"...\" style:\"grad\"\"", "Code lines:[\"...\"]", "Video src:\"https://...\"", "Candlestick data:signal stream:\"/api/...\"", "LineChart data:signal series:signal", "AreaChart data:signal series:signal", "BarChart data:signal", "ArcChart data:signal", "PieChart data:signal", "Table data:signal column field:\"name\" label:\"Name\"", "Tabs tab children", "Fab fabAction children", "ComboBox comboOption children", "CsvField csvColumn children", "DragDrop dragItem dragGroup children", "NavMenu item submenu megamenu children", "Scaffold appBar start main end bottomBar regions", "Modal header footer regions", "Drawer header body footer regions", "Dropdown trigger item divider regions", "Command group item entries", "Marquee children", "TypeWriter item text:\"...\"", "Collapsible label:\"Details\" children", "Map marker id:\"office\" lat:4.71 lng:-74.07", "Divider orientation:\"horizontal\"", "Svg Path children", "AppBar/Footer/BottomBar start center end regions"],
  "standardLibrary": ["str.trim", "str.lower", "str.upper", "str.length", "math.sum", "parse.int", "parse.float", "parse.json", "url.parse", "url.querySet", "csv.parse", "sort.by", "list.filterContains", "json.get", "json.stringify", "date.now"],
  "signalPathValidation": "known object fields and supported target scalar type are checked before generation",
  "declaredTypes": ["signal form type:Form value:{ ... }", "signal rows type:Row[] value:[]"],
  "staticStrings": ["Text i18n:\"home.hero.summary\"", "Title i18n:\"home.hero.title\"", "RichText i18n:\"home.hero.summary\"", "RichText mark text:\"Launch\" style:\"grad\" scheme:\"primary\"", "Input label:\"...\"", "Select placeholder:\"...\"", "Option value:\"...\"", "ComboBox placeholder:\"Choose\" searchPlaceholder:\"Search\"", "comboOption value:\"admin\" label:\"Administrator\"", "CsvField buttonText:\"Upload CSV\" modalTitle:\"Review import\"", "csvColumn name:\"email\" label:\"Email\"", "DragDrop direction:\"horizontal\" emptyText:\"No items\"", "dragGroup id:\"todo\" title:\"Todo\"", "dragItem id:\"draft\" label:\"Draft\"", "Editor placeholder:\"Write notes\"", "ImageCropper shape:\"circle\"", "PasswordField weakLabel:\"Weak\" mediumLabel:\"Medium\" strongLabel:\"Strong\"", "PhoneField country:\"US\"", "PinField type:\"number\"", "Textarea rows:4 maxLength:160", "Slider label:\"Volume\" min:0 max:100 step:5", "Dropzone accept:\"image/*\" placeholder:\"Drop files\"", "ToggleTheme lightLabel:\"Light mode\" darkLabel:\"Dark mode\"", "Fab position:\"bottom-right\" icon:\"plus\"", "fabAction label:\"Docs\" icon:\"link\" href:\"/docs\"", "Record name:\"voice\" maxDuration:90 variant:\"soft\"", "ToggleGroup selected:\"map\" size:\"sm\" ariaLabel:\"Display mode\"", "ToggleGroup item id:\"map\" label:\"Map\"", "Collapsible label:\"Details\" defaultOpen:true", "Countdown target:\"2030-01-01T00:00:00Z\" size:\"md\"", "Map centerLat:4.7109 centerLng:-74.0721 zoom:12 height:\"360px\"", "Map marker id:\"office\" label:\"Office\" icon:\"start\"", "Code language:\"dowe\"", "Video aspect:\"horizontal\"", "Candlestick emptyLabel:\"No candle data\"", "LineChart curve:\"smooth\" palette:\"ocean\"", "AreaChart fillOpacity:0.3 legendPosition:\"bottom\"", "BarChart grouped:true size:\"lg\"", "ArcChart startAngle:-90 endAngle:270", "PieChart donut:true donutWidth:72", "Table scheme:\"surface\"", "Table emptyTitle:\"No data\"", "column field:\"name\" label:\"Name\" align:\"start\"", "Tabs variant:\"line\"", "Tabs scheme:\"primary\"", "Tabs position:\"top\"", "tab id:\"overview\" label:\"Overview\"", "NavMenu scheme:\"muted\"", "NavMenu item label:\"Docs\" href:\"/docs\"", "Sidebar body SideNav", "Scaffold boxed:true", "Divider scheme:\"muted\"", "Path fill:\"none\"", "AppBar scheme:\"surface\"", "Footer scheme:\"background\"", "BottomBar variant:\"soft\"", "Drawer position:\"start\"", "Avatar name:\"Ada\" size:\"md\" status:\"online\"", "AvatarGroup size:\"md\" max:4 inline:false", "AvatarGroup item src:\"/team/ada.png\" name:\"Ada\"", "ChatBox mode:\"conversation\" placeholder:\"Type a message...\"", "Empty type:\"data\" title:\"No data\" actionLabel:\"View more\"", "Marquee speed:\"normal\" orientation:\"horizontal\" fade:true", "TypeWriter typeSpeed:100 deleteSpeed:50 item text:\"Build systems\"", "Badge text:\"3\" position:\"top-right\"", "Chip variant:\"soft\" size:\"sm\"", "Skeleton variant:\"rounded\" animation:\"wave\"", "Modal scheme:\"surface\"", "AlertDialog title:\"Are you sure?\"", "Tooltip label:\"More\" position:\"top\"", "Toast type:\"success\" position:\"bottom-left\"", "Dropdown item label:\"Docs\" href:\"/docs\"", "Command group label:\"Navigation\"", "Box animation:\"fadeIn\"", "Section background:\"aurora\"", "Section background values: soft aurora sunrise ocean meadow slate", "Card animation:\"slideUp\""],
  "visibility": ["show:true", "show:false", "show:{ xs:false md:true }", "show:signalBool"],
  "controlFlow": {
    "if": "static true and false conditions are supported until data surfaces are specified",
    "each": "renders signal arrays with deterministic key paths"
  },
  "navigation": ["href", "navigate", "history", "target", "externalMode"],
  "serverApisAvailable": false
}
"#;

const I18N_SURFACE: &str = r#"{
  "root": "src/i18n/<locale>.dowe",
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
  "root": "src/config.dowe",
  "rootBlock": "config",
  "blocks": ["config", "app", "fonts", "design", "theme", "colors", "radii", "env", "variable", "server", "cors"],
  "app": {
    "declaration": "app name:\"Dowe Dev\" bundle:\"dev.dowe.generated\"",
    "bundle": "reverse-dns"
  },
  "fonts": ["system", "inter", "roboto", "montserrat", "lato", "poppins", "manrope", "quicksand", "lora"],
  "themes": {
    "defaultTheme": "light",
    "builtInInheritance": ["light", "dark"],
    "runtimeSwitching": false
  },
  "colorValues": ["#RGB", "#RRGGBB", "#RRGGBBAA"],
  "radii": ["radius", "radiusBox", "radiusUi"],
  "environment": {
    "declaration": "variable name:\"BACKEND_URL\" visibility:\"client\" required:false default:\"\"",
    "visibility": ["server", "client"],
    "resolutionOrder": [".env", "operating-system", "default"],
    "clientValuesArePublic": true
  },
  "cors": {
    "declaration": "cors target:\"server\" devOrigins:true origins:[\"http://127.0.0.1:56035\"] methods:[\"GET\",\"POST\",\"PATCH\",\"DELETE\"] headers:[\"Content-Type\"] credentials:false maxAge:600",
    "targets": ["server", "desktop", "all"],
    "origins": ["exact-http-origin", "exact-https-origin", "*"],
    "devOrigins": "dowe-dev-managed-client-origins",
    "credentialsWithWildcard": false
  },
  "obsoleteConfig": "dowe.json"
}
"##;
