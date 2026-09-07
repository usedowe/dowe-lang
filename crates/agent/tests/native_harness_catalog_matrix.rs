use dowe_agent::native_harness::select_units;

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
