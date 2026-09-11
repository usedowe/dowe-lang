use dowe_agent::native_harness::{select_units, skill_unit};

#[test]
fn equivalent_english_and_spanish_intents_select_the_same_focused_units() {
    for (english, spanish, expected) in [
        ("Describe the system", "Describe el sistema", vec!["core"]),
        (
            "Define the business domain",
            "Define el dominio de negocio",
            vec!["core", "domain-modeling"],
        ),
        ("Review IPC", "Revisa IPC", vec!["core", "native-ipc"]),
        (
            "Adjust theme colors",
            "Ajusta los colores del tema",
            vec!["core", "theme"],
        ),
        (
            "Update environment configuration",
            "Actualiza la configuración del entorno",
            vec!["core", "core/configuration"],
        ),
        (
            "Run tests",
            "Ejecuta las pruebas",
            vec!["core", "core/validation"],
        ),
        (
            "Adjust the sidebar",
            "Ajusta la barra lateral",
            vec!["core", "views/layouts"],
        ),
        (
            "Create a page",
            "Crea una página",
            vec!["core", "views/pages"],
        ),
        (
            "Extract components",
            "Extrae componentes",
            vec!["core", "views/components"],
        ),
        (
            "Add entities",
            "Agrega entidades",
            vec!["core", "server/entities"],
        ),
        (
            "Correct the handler",
            "Corrige el handler",
            vec!["core", "server/handlers"],
        ),
        (
            "Extract functions",
            "Extrae funciones",
            vec!["core", "server/functions"],
        ),
        ("Add routes", "Agrega rutas", vec!["core", "server/routes"]),
        (
            "Configure the database",
            "Configura la base de datos",
            vec!["core", "server/persistence"],
        ),
        (
            "Connect the request to an endpoint",
            "Conecta la solicitud a un endpoint",
            vec!["core", "server/handlers", "server/routes", "views/requests"],
        ),
    ] {
        assert_eq!(select_units(english, &[]), expected, "{english}");
        assert_eq!(select_units(spanish, &[]), expected, "{spanish}");
    }
}

#[test]
fn substrings_do_not_expand_unrelated_knowledge_and_paths_take_precedence() {
    assert_eq!(
        select_units("Requested theme update", &[]),
        vec!["core", "theme"]
    );
    assert_eq!(select_units("Describe el sistema", &[]), vec!["core"]);
    assert_eq!(
        select_units("Inspect ./server/entities/theme-settings.dowe", &[]),
        vec!["core", "server/entities"]
    );
    assert_eq!(
        select_units("Inspect server/entities/theme-settings.dowe", &[]),
        vec!["core", "server/entities"]
    );
    assert_eq!(
        select_units("Inspect `main.dowe`.", &[]),
        vec!["core", "core/configuration"]
    );
    assert_eq!(
        select_units("Inspect .gitignore", &[]),
        vec!["core", "core/configuration"]
    );
}

#[test]
fn reference_image_intents_preload_the_complete_view_composition_contract() {
    let units = select_units("implementa esta imagen de referencia en Dowe", &[]);
    assert_eq!(
        units,
        vec!["core", "views/components", "views/layouts", "views/pages"]
    );
}

#[test]
fn ui_and_design_intents_preload_the_complete_view_composition_contract() {
    for prompt in ["build the UI", "design a dashboard"] {
        let units = select_units(prompt, &[]);
        assert!(units.iter().any(|unit| unit == "views/layouts"), "{prompt}");
        assert!(units.iter().any(|unit| unit == "views/pages"), "{prompt}");
        assert!(
            units.iter().any(|unit| unit == "views/components"),
            "{prompt}"
        );
    }
}

#[test]
fn focused_view_units_carry_default_first_guidance() {
    for id in ["views/layouts", "views/pages", "views/components"] {
        let unit = skill_unit(id).unwrap();
        assert!(
            unit.content
                .contains("component design defaults are the visual baseline")
        );
        assert!(
            unit.content.to_ascii_lowercase().contains("omit redundant"),
            "{id}"
        );
    }
}
