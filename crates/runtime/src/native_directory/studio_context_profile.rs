pub(super) fn infer_studio_profile(
    query: &str,
    entries: &BTreeMap<String, StudioSourceEntry>,
    has_image: bool,
) -> &'static str {
    if has_image {
        return "viewReference";
    }
    let lower = query.to_ascii_lowercase();
    let views_request = contains_studio_term(
        &lower,
        &[
            "ui",
            "ux",
            "frontend",
            "dashboard",
            "view",
            "vista",
            "layout",
            "page",
            "screen",
            "pantalla",
            "component",
            "componente",
            "form",
            "formulario",
            "button",
            "responsive",
        ],
    );
    let server_request = contains_studio_term(
        &lower,
        &[
            "backend",
            "server",
            "servidor",
            "api",
            "endpoint",
            "handler",
            "middleware",
            "database",
            "persistencia",
            "cache",
            "vector",
            "websocket",
            "provider",
        ],
    );
    if contains_studio_term(
        &lower,
        &[
            "diagnostic",
            "diagnóstico",
            "compiler error",
            "compile error",
            "error de compilación",
        ],
    ) {
        return "diagnostics";
    }
    if contains_studio_term(
        &lower,
        &[
            "screenshot",
            "mockup",
            "reference",
            "referencia",
            "imagen",
            "captura",
        ],
    ) {
        return "viewReference";
    }
    if contains_studio_term(
        &lower,
        &["table", "tabla", "datagrid", "pagination", "paginación"],
    ) {
        return "viewTable";
    }
    if contains_studio_term(
        &lower,
        &["theme", "tema", "palette", "paleta", "colors", "colores"],
    ) {
        return "theme";
    }
    if contains_studio_term(
        &lower,
        &[
            "ipc",
            "filesystem",
            "file system",
            "native host",
            "staging",
            "preview host",
        ],
    ) {
        return "native";
    }
    if contains_studio_term(
        &lower,
        &[
            "pos",
            "point of sale",
            "inventory",
            "inventario",
            "checkout",
        ],
    ) {
        return "domainPos";
    }
    if contains_studio_term(
        &lower,
        &["crm", "pipeline", "lead", "opportunity", "cliente"],
    ) {
        return "domainCrm";
    }
    if contains_studio_term(
        &lower,
        &[
            "ecommerce",
            "e-commerce",
            "catalog",
            "catálogo",
            "cart",
            "carrito",
            "fulfillment",
        ],
    ) {
        return "domainEcommerce";
    }
    if contains_studio_term(
        &lower,
        &[
            "reservation",
            "reservations",
            "reserva",
            "reservas",
            "booking",
            "booking system",
        ],
    ) {
        return "domainReservations";
    }
    if contains_studio_term(
        &lower,
        &["domain", "dominio", "workflow", "workflow de negocio"],
    ) {
        return "domain";
    }
    if views_request && server_request {
        return "fullstack";
    }
    if server_request {
        return "server";
    }
    if views_request {
        return "views";
    }
    let has_views = entries.keys().any(|path| path.starts_with("views/"));
    let has_server = entries.keys().any(|path| path.starts_with("server/"));
    match (has_views, has_server) {
        (true, true) => "fullstack",
        (false, true) => "server",
        (true, false) => "views",
        (false, false) => "core",
    }
}

pub(super) fn infer_studio_request_type(query: &str, has_image: bool) -> &'static str {
    if has_image {
        return "vision_ui";
    }
    let lower = query.to_ascii_lowercase();
    if contains_studio_term(
        &lower,
        &[
            "read",
            "inspect",
            "explore",
            "context",
            "analiza",
            "analizar",
            "inspecciona",
        ],
    ) {
        "context_read"
    } else if contains_studio_term(&lower, &["review", "revisar", "diff", "audit", "auditar"]) {
        "review"
    } else if contains_studio_term(
        &lower,
        &[
            "validate",
            "validation",
            "validar",
            "test",
            "tests",
            "prueba",
        ],
    ) {
        "validation"
    } else if contains_studio_term(
        &lower,
        &["plan", "spec", "design", "diseña", "arquitectura"],
    ) {
        "spec_plan"
    } else if contains_studio_term(
        &lower,
        &[
            "implement",
            "implementation",
            "fix",
            "corrige",
            "arregla",
            "write",
            "escribe",
        ],
    ) {
        "implementation"
    } else if contains_studio_term(
        &lower,
        &[
            "screenshot",
            "mockup",
            "reference",
            "referencia",
            "imagen",
            "captura",
        ],
    ) {
        "vision_ui"
    } else if query.split_whitespace().count() <= 4 {
        "clarify"
    } else {
        "spec_plan"
    }
}

pub(super) fn contains_studio_term(value: &str, terms: &[&str]) -> bool {
    let words = value
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|word| !word.is_empty())
        .collect::<BTreeSet<_>>();
    terms.iter().any(|term| {
        if term.contains(' ') || term.contains('-') {
            value.contains(term)
        } else {
            words.contains(term)
        }
    })
}

pub(super) fn parse_studio_imports(source: &str, importer: &str) -> Vec<String> {
    let parent = Path::new(importer)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let mut imports = BTreeSet::new();
    for line in source.lines() {
        let Some(index) = line.find(" from \"") else {
            continue;
        };
        let rest = &line[index + 7..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let specifier = &rest[..end];
        let candidate = if let Some(path) = specifier.strip_prefix("@/") {
            PathBuf::from(path)
        } else if let Some(path) = specifier.strip_prefix("./") {
            parent.join(path)
        } else {
            continue;
        };
        let Some(relative) = normalize_studio_source_path(&candidate) else {
            continue;
        };
        imports.insert(relative);
    }
    imports.into_iter().collect()
}

pub(super) fn normalize_studio_source_path(path: &Path) -> Option<String> {
    if path.is_absolute() {
        return None;
    }
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return None;
        };
        components.push(value.to_string_lossy().into_owned());
    }
    if components.is_empty() {
        return None;
    }
    let mut relative = components.join("/");
    if !relative.ends_with(".dowe") {
        relative.push_str(".dowe");
    }
    Some(relative)
}

pub(super) fn parse_studio_declarations(source: &str, path: &str) -> Vec<String> {
    const DECLARATIONS: &[&str] = &[
        "app",
        "component",
        "database",
        "desktop",
        "endpoints",
        "entity",
        "fn",
        "group",
        "handler",
        "layout",
        "main",
        "middleware",
        "page",
        "route",
        "server",
        "store",
        "theme",
        "type",
        "views",
        "websocket",
    ];
    source
        .lines()
        .filter_map(|line| {
            if line
                .chars()
                .next()
                .is_some_and(|character| character.is_whitespace())
            {
                return None;
            }
            let mut words = line.split_whitespace();
            let declaration = words.next()?;
            if !DECLARATIONS.contains(&declaration) {
                return None;
            }
            let name = words
                .next()
                .unwrap_or_default()
                .trim_end_matches(':')
                .trim_end_matches('"');
            Some(format!("{path}:{declaration}:{name}"))
        })
        .collect()
}


