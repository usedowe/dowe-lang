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
  "propStrings": "quoted static literals; bare values are validated references only"
}
"#;

const SERVER_SURFACE: &str = r#"{
  "root": "src/main.dowe",
  "blocks": ["type", "main", "server", "desktop", "route", "websocket", "init", "handler", "middleware"],
  "httpMethods": ["GET", "POST", "PUT", "DELETE", "PATCH"],
  "actions": ["let", "return", "return continue", "response text", "response json", "log", "info", "warn", "error"],
  "request": ["req.params", "req.json()", "req.header", "req.context"],
  "middleware": ["route middleware:[name]", "middleware name async req", "bearer", "jwt.verify", "jwt.decrypt", "jwt.sign", "jwt.encrypt"],
  "jwt": {
    "jws": ["HS256"],
    "jwe": ["dir", "A256GCM"],
    "secrets": "server env variables only"
  },
  "inferredReferences": ["store.insert field", "store.insert id", "store.update changed", "store.delete changed"],
  "declaredTypes": ["let body:Type = await req.json()", "validated request JSON", "typed body references"],
  "resolvedLogValues": true,
  "handlers": ["method GET handler:name", "method POST handler:name", "method PATCH handler:name", "method DELETE handler:name"],
  "websocketHandlers": ["open", "message", "close", "drain"],
  "hostRuntime": "rust",
  "nodeRuntime": false
}
"#;

const VIEWS_SURFACE: &str = r#"{
  "root": "src/views.dowe",
  "exports": ["views", "layout", "page", "type"],
  "components": ["Box", "Section", "Flex", "Grid", "Input", "Select", "Option", "Code", "Video", "Candlestick", "Table", "Divider", "Button", "Alert", "Svg", "Path", "Card", "AppBar", "Footer", "BottomBar", "SideNav", "Sidebar", "NavMenu", "Scaffold", "Tabs", "tab", "Drawer", "Avatar", "Badge", "Chip", "Skeleton", "Modal", "AlertDialog", "Tooltip", "Toast", "Dropdown", "Command", "Title", "Text"],
  "slots": ["children", "start", "center", "end", "appBar", "main", "bottomBar", "content", "header", "footer", "trigger", "icon", "group", "item", "divider"],
  "routing": ["route path:\"/\" layout:Layout platform:\"web\"", "route path:\"/\" layout:Layout platform:[\"desktop\",\"ios\",\"android\"]", "page path:\"\" component:Page platform:\"desktop\"", "platform values: web desktop android ios"],
  "reactivity": ["signal", "action", "request route:\"/api/...\"", "request path:\"/...\"", "request base:env.NAME", "onSuccess alert:\"...\"", "onError alert:\"...\"", "implicit BACKEND_URL for /api", "assign", "reset", "Input bind:signal.field", "Select bind:signal.field", "Button onClick:action", "Avatar onClick:action", "Chip onClose:action", "Modal onClose:action open:signalBool", "AlertDialog onConfirm:action onCancel:action open:signalBool", "Dropdown item onClick:action", "Command open:signalBool item onClick:action", "Toast source:signalObject", "show:signalBool", "Drawer open:signalBool", "Button text child", "Title text child", "Text text child", "Code lines:[\"...\"]", "Video src:\"https://...\"", "Candlestick data:signal stream:\"/api/...\"", "Table data:signal column field:\"name\" label:\"Name\"", "Tabs tab children", "NavMenu item submenu megamenu children", "Scaffold appBar start main end bottomBar regions", "Modal header footer regions", "Dropdown trigger item divider regions", "Command group item entries", "Divider orientation:\"horizontal\"", "Svg Path children", "AppBar/Footer/BottomBar start center end regions"],
  "signalPathValidation": "known object fields and supported target scalar type are checked before generation",
  "declaredTypes": ["signal form type:Form value:{ ... }", "signal rows type:Row[] value:[]"],
  "staticStrings": ["Text i18n:\"home.hero.summary\"", "Title i18n:\"home.hero.title\"", "Input label:\"...\"", "Select placeholder:\"...\"", "Option value:\"...\"", "Code language:\"dowe\"", "Video aspect:\"horizontal\"", "Candlestick emptyLabel:\"No candle data\"", "Table scheme:\"surface\"", "Table emptyTitle:\"No data\"", "column field:\"name\" label:\"Name\" align:\"start\"", "Tabs variant:\"line\"", "Tabs scheme:\"primary\"", "Tabs position:\"top\"", "tab id:\"overview\" label:\"Overview\"", "NavMenu scheme:\"muted\"", "NavMenu item label:\"Docs\" href:\"/docs\"", "Sidebar scheme:\"muted\"", "Scaffold boxed:true", "Divider scheme:\"muted\"", "Path fill:\"none\"", "AppBar scheme:\"surface\"", "Footer scheme:\"background\"", "BottomBar variant:\"soft\"", "Drawer position:\"start\"", "Avatar name:\"Ada\" size:\"md\" status:\"online\"", "Badge text:\"3\" position:\"top-right\"", "Chip variant:\"soft\" size:\"sm\"", "Skeleton variant:\"rounded\" animation:\"wave\"", "Modal scheme:\"surface\"", "AlertDialog title:\"Are you sure?\"", "Tooltip label:\"More\" position:\"top\"", "Toast type:\"success\" position:\"bottom-left\"", "Dropdown item label:\"Docs\" href:\"/docs\"", "Command group label:\"Navigation\"", "Box animation:\"fadeIn\"", "Section background:\"aurora\"", "Section background values: soft aurora sunrise ocean meadow slate", "Card animation:\"slideUp\""],
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
  "components": ["Text", "Title"],
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
