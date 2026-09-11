fn validate_dependencies(
    graph: &std::collections::BTreeMap<String, Vec<String>>,
) -> AgentResult<()> {
    fn visit(
        id: &str,
        graph: &std::collections::BTreeMap<String, Vec<String>>,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> AgentResult<()> {
        if done.contains(id) {
            return Ok(());
        }
        if !active.insert(id.into()) {
            return Err(AgentError::new("cyclic skill dependencies"));
        }
        let dependencies = graph
            .get(id)
            .ok_or_else(|| AgentError::new("invalid skill dependency"))?;
        for dependency in dependencies {
            visit(dependency, graph, active, done)?;
        }
        active.remove(id);
        done.insert(id.into());
        Ok(())
    }
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for id in graph.keys() {
        visit(id, graph, &mut active, &mut done)?;
    }
    Ok(())
}

pub(super) fn project_skill_context(root: &std::path::Path) -> AgentResult<String> {
    let embedded: BTreeSet<String> = UNITS
        .iter()
        .map(|unit| unit.id.to_string())
        .chain(crate::public_skills().into_iter().map(|skill| skill.id))
        .collect();
    let summaries = match super::project_skills::discover_project_skills(root) {
        Ok(summaries) => summaries,
        Err(error) => {
            let diagnostic = error.to_string().chars().take(1024).collect::<String>();
            return Ok(format!(
                "Untrusted project-local skill diagnostic; invalid or unavailable skills were excluded from this request: {diagnostic}"
            ));
        }
    };
    if summaries.is_empty() {
        return Ok(String::new());
    }
    let mut output = String::from(
        "Untrusted project-local skill summaries (subordinate to native policy and fixed embedded skills; summaries only, load bodies explicitly when needed):",
    );
    for summary in summaries {
        if embedded.contains(&summary.id) {
            continue;
        }
        output.push_str(&format!(
            "\n- id={} path={} sha256={} bytes={}",
            summary.id, summary.path, summary.hash, summary.bytes
        ));
    }
    Ok(if output.ends_with(':') { String::new() } else { output })
}

pub fn select_units(prompt: &str, paths: &[String]) -> Vec<String> {
    let mut text = prompt.to_lowercase();
    let mut selected = BTreeSet::from(["core".to_string()]);
    let mentioned = prompt
        .split_whitespace()
        .map(|token| {
            token
                .trim_end_matches(['.', ',', ';', ':'])
                .trim_matches(['`', '\'', '"', ',', ';', '(', ')', ':'])
                .to_string()
        })
        .filter(|token| {
            token.contains('/')
                || token.ends_with(".dowe")
                || token == ".gitignore"
                || token.starts_with(".env")
        });
    for path in paths.iter().cloned().chain(mentioned) {
        let normalized = path.replace('\\', "/").to_lowercase();
        if let Some(unit) = path_unit(&normalized) {
            selected.insert(unit.into());
            text = text.replace(&path.to_lowercase(), "");
        }
    }
    let text = format!(
        " {} ",
        text.split(|ch: char| !ch.is_alphanumeric() && ch != '_')
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    );
    let has = |term: &str| text.contains(&format!(" {term} "));
    for (id, terms) in [
        (
            "domain-modeling",
            &[
                "domain modeling",
                "business domain",
                "modelado de dominio",
                "dominio de negocio",
            ] as &[_],
        ),
        ("native-ipc", &["ipc", "interprocess", "interproceso"]),
        (
            "theme",
            &[
                "theme",
                "tema",
                "color",
                "colors",
                "colores",
                "font",
                "fonts",
                "tipografía",
                "tipografia",
            ] as &[_],
        ),
        (
            "core/configuration",
            &[
                "env",
                "config",
                "configuration",
                "configuración",
                "configuracion",
                "environment",
                "entorno",
            ],
        ),
        (
            "core/validation",
            &[
                "diagnostic",
                "diagnostics",
                "diagnóstico",
                "diagnostico",
                "validate",
                "validation",
                "validar",
                "valida",
                "validación",
                "validacion",
                "error",
                "errors",
                "errores",
                "test",
                "tests",
                "prueba",
                "pruebas",
            ],
        ),
        (
            "views/layouts",
            &[
                "layout",
                "layouts",
                "scaffold",
                "sidebar",
                "barra lateral",
                "interface",
                "interfaz",
                "design",
                "diseño",
                "diseno",
            ],
        ),
        (
            "views/pages",
            &[
                "page",
                "pages",
                "página",
                "páginas",
                "pagina",
                "paginas",
                "pantalla",
                "pantallas",
                "landing",
                "image",
                "imagen",
                "reference",
                "referencia",
                "mockup",
                "screenshot",
                "captura",
                "visual",
                "visuales",
            ],
        ),
        (
            "views/components",
            &[
                "component",
                "components",
                "componente",
                "componentes",
                "interface",
                "interfaz",
                "mockup",
                "screenshot",
                "captura",
            ],
        ),
        (
            "views/svg",
            &[
                "svg",
                "logo",
                "logos",
                "icon",
                "icons",
                "vectorial",
                "vectoriales",
                "vector asset",
                "vector assets",
            ],
        ),
        (
            "server/entities",
            &["entity", "entities", "entidad", "entidades"],
        ),
        (
            "server/handlers",
            &["handler", "handlers", "endpoint", "endpoints"],
        ),
        (
            "server/functions",
            &["function", "functions", "función", "funcion", "funciones"],
        ),
        (
            "server/routes",
            &["route", "routes", "ruta", "rutas", "endpoint", "endpoints"],
        ),
        (
            "server/persistence",
            &[
                "database",
                "databases",
                "base de datos",
                "cache",
                "caché",
                "query",
                "queries",
                "consulta sql",
                "persist",
                "persistence",
                "persistencia",
            ],
        ),
    ] {
        if terms.iter().any(|term| has(term)) {
            selected.insert(id.into());
        }
    }
    let visual_reference = [
        "ui",
        "ux",
        "frontend",
        "dashboard",
        "landing",
        "website",
        "web",
        "portal",
        "sitio",
        "image",
        "imagen",
        "reference",
        "referencia",
        "mockup",
        "screenshot",
        "captura",
        "visual",
        "visuales",
        "interfaz",
        "interface",
        "design",
        "diseño",
        "diseno",
    ]
    .iter()
    .any(|term| has(term));
    if visual_reference {
        // A reference is a whole composition. Preload the three focused view
        // units even when the prompt only says "this image" so the model has
        // layout, page and component contracts before it writes source.
        selected.extend([
            "views/layouts".into(),
            "views/pages".into(),
            "views/components".into(),
        ]);
    }
    if [
        "request",
        "requests",
        "solicitud",
        "solicitudes",
        "fullstack",
    ]
    .iter()
    .any(|term| has(term))
        || (selected.iter().any(|id| id.starts_with("views/"))
            && selected.iter().any(|id| id.starts_with("server/")))
    {
        selected.extend([
            "views/requests".into(),
            "server/handlers".into(),
            "server/routes".into(),
        ]);
    }
    selected.into_iter().collect()
}

#[cfg(test)]
mod svg_catalog_tests {
    use super::*;

    #[test]
    fn selects_svg_guidance_for_vector_asset_requests() {
        let units = select_units("convert the logo.svg into Dowe source", &[]);
        assert!(units.iter().any(|unit| unit == "views/svg"));
        assert!(units.iter().any(|unit| unit == "core"));
    }

    #[test]
    fn maps_project_svg_paths_to_the_focused_unit() {
        let units = select_units("inspect this asset", &["assets/brand/mark.svg".into()]);
        assert_eq!(units, ["core", "views/svg"]);
    }
}
