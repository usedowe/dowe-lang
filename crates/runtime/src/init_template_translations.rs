const BLANK_TRANSLATIONS: &[InitTranslation] = &[
    InitTranslation {
        key: "brand.name",
        en: "DOWE",
        es: "DOWE",
    },
    InitTranslation {
        key: "blank.title",
        en: "Hello Dowe",
        es: "Hola Dowe",
    },
    InitTranslation {
        key: "blank.description",
        en: "Your first page and Rust server endpoint are ready.",
        es: "Tu primera página y el endpoint del servidor Rust están listos.",
    },
];

const CRUD_TRANSLATIONS: &[InitTranslation] = &[
    InitTranslation {
        key: "brand.title",
        en: "DOWE JOURNAL",
        es: "DIARIO DOWE",
    },
    InitTranslation {
        key: "brand.subtitle",
        en: "Editorial workspace",
        es: "Espacio editorial",
    },
    InitTranslation {
        key: "hero.title",
        en: "Stories worth returning to.",
        es: "Historias a las que vale la pena volver.",
    },
    InitTranslation {
        key: "hero.description",
        en: "A polished fullstack journal with portable views, protected writing, and Rust-owned data.",
        es: "Un diario fullstack cuidado, con views portables, escritura protegida y datos controlados por Rust.",
    },
    InitTranslation {
        key: "actions.createAccount",
        en: "Create account",
        es: "Crear cuenta",
    },
    InitTranslation {
        key: "actions.signIn",
        en: "Sign in",
        es: "Iniciar sesión",
    },
    InitTranslation {
        key: "actions.newStory",
        en: "New story",
        es: "Nueva historia",
    },
    InitTranslation {
        key: "actions.editStory",
        en: "Edit my story",
        es: "Editar mi historia",
    },
    InitTranslation {
        key: "session.eyebrow",
        en: "CURRENT SESSION",
        es: "SESIÓN ACTUAL",
    },
    InitTranslation {
        key: "session.guestTitle",
        en: "Guest workspace",
        es: "Espacio de invitado",
    },
    InitTranslation {
        key: "session.guestDescription",
        en: "Sign in to publish and maintain your own stories.",
        es: "Inicia sesión para publicar y mantener tus propias historias.",
    },
    InitTranslation {
        key: "session.readyDescription",
        en: "Authenticated and ready to publish.",
        es: "Autenticado y listo para publicar.",
    },
    InitTranslation {
        key: "journal.eyebrow",
        en: "COMMUNITY JOURNAL",
        es: "DIARIO DE LA COMUNIDAD",
    },
    InitTranslation {
        key: "journal.title",
        en: "Latest stories",
        es: "Historias recientes",
    },
    InitTranslation {
        key: "journal.refresh",
        en: "Refresh",
        es: "Actualizar",
    },
    InitTranslation {
        key: "loading.session",
        en: "Validating your session",
        es: "Validando tu sesión",
    },
    InitTranslation {
        key: "loading.blogs",
        en: "Loading the latest stories",
        es: "Cargando las historias más recientes",
    },
    InitTranslation {
        key: "controls.eyebrow",
        en: "QUICK ACTIONS",
        es: "ACCIONES RÁPIDAS",
    },
    InitTranslation {
        key: "controls.title",
        en: "Editorial controls",
        es: "Controles editoriales",
    },
    InitTranslation {
        key: "controls.description",
        en: "Focused tasks open in their own workspace without crowding the journal.",
        es: "Cada tarea se abre en su propio espacio sin saturar el diario.",
    },
    InitTranslation {
        key: "controls.join",
        en: "Join the journal",
        es: "Unirme al diario",
    },
    InitTranslation {
        key: "controls.access",
        en: "Access my account",
        es: "Acceder a mi cuenta",
    },
    InitTranslation {
        key: "controls.publish",
        en: "Publish a story",
        es: "Publicar una historia",
    },
    InitTranslation {
        key: "controls.revise",
        en: "Revise a story",
        es: "Revisar una historia",
    },
    InitTranslation {
        key: "security.eyebrow",
        en: "SECURE BY DESIGN",
        es: "SEGURO POR DISEÑO",
    },
    InitTranslation {
        key: "security.title",
        en: "Your stories stay yours.",
        es: "Tus historias siguen siendo tuyas.",
    },
    InitTranslation {
        key: "security.description",
        en: "The verified session subject is the only owner accepted by create and update.",
        es: "El subject de sesión verificado es el único propietario aceptado al crear y actualizar.",
    },
    InitTranslation {
        key: "register.eyebrow",
        en: "NEW WRITER",
        es: "NUEVO AUTOR",
    },
    InitTranslation {
        key: "register.title",
        en: "Create your account",
        es: "Crea tu cuenta",
    },
    InitTranslation {
        key: "register.description",
        en: "Join the journal to publish stories and maintain your own work.",
        es: "Únete al diario para publicar historias y mantener tu propio trabajo.",
    },
    InitTranslation {
        key: "common.cancel",
        en: "Cancel",
        es: "Cancelar",
    },
    InitTranslation {
        key: "login.eyebrow",
        en: "WRITER ACCESS",
        es: "ACCESO DE AUTOR",
    },
    InitTranslation {
        key: "login.title",
        en: "Welcome back",
        es: "Bienvenido de nuevo",
    },
    InitTranslation {
        key: "login.description",
        en: "Continue writing with your protected editorial session.",
        es: "Continúa escribiendo con tu sesión editorial protegida.",
    },
    InitTranslation {
        key: "create.eyebrow",
        en: "NEW STORY",
        es: "NUEVA HISTORIA",
    },
    InitTranslation {
        key: "create.title",
        en: "Publish something memorable",
        es: "Publica algo memorable",
    },
    InitTranslation {
        key: "create.description",
        en: "Give the community a clear title and a thoughtful story.",
        es: "Comparte con la comunidad un título claro y una historia bien pensada.",
    },
    InitTranslation {
        key: "create.saveLater",
        en: "Save for later",
        es: "Guardar para después",
    },
    InitTranslation {
        key: "create.publish",
        en: "Publish story",
        es: "Publicar historia",
    },
    InitTranslation {
        key: "edit.eyebrow",
        en: "OWNER EDIT",
        es: "EDICIÓN DEL PROPIETARIO",
    },
    InitTranslation {
        key: "edit.title",
        en: "Refine your story",
        es: "Mejora tu historia",
    },
    InitTranslation {
        key: "edit.description",
        en: "Use the story id from the journal. The backend verifies ownership before saving.",
        es: "Usa el id de la historia del diario. El backend verifica la propiedad antes de guardar.",
    },
    InitTranslation {
        key: "edit.save",
        en: "Save changes",
        es: "Guardar cambios",
    },
];


