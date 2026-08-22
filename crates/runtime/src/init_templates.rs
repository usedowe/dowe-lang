use crate::init::{InitProjectOptions, ProjectTemplate, TemplateFile};

mod crud_server;

use crud_server::{
    CRUD_API_ROUTES, CRUD_AUTH_MIDDLEWARE, CRUD_AUTH_TYPES, CRUD_BLOG_TYPES, CRUD_BLOGS,
    CRUD_BLOGS_HANDLER, CRUD_BLOGS_REPOSITORY, CRUD_BLOGS_SERVICE, CRUD_DATABASE, CRUD_SESSIONS,
    CRUD_USERS, CRUD_USERS_HANDLER, CRUD_USERS_REPOSITORY, CRUD_USERS_SERVICE,
};

const GITIGNORE: &str = ".dowe\n.env\n.env.live\n.env.stage\n.env.uat\n";

const ZED_SETTINGS: &str = r#"{
  "languages": {
    "Dowe": {
      "formatter": "language_server",
      "format_on_save": "on",
      "preferred_line_length": 100
    }
  }
}
"#;

const BLANK_THEME: &str = r##"theme
  fonts default:"inter" install:["inter"]
  design defaultTheme:"light"
    theme name:"light"
      colors:
        primary color:"#1F3A5F" text:"#EAF2F8" title:"#FFFFFF"
        secondary color:"#6BC670" text:"#102A15" title:"#071B0B"
        background color:"#FFFFFF" text:"#17263A" title:"#17263E"
        surface color:"#F7F9FC" text:"#17263A" title:"#17263E"
"##;

const CRUD_THEME: &str = r##"theme
  fonts default:"manrope" install:["manrope"]
  design defaultTheme:"editorial"
    Card variant:"solid" scheme:"surface" rounded:"xl" shadow:"xs" shadowColor:"muted"
    Button variant:"solid" scheme:"primary" size:"md" rounded:"full"
    Avatar variant:"solid" scheme:"primary" rounded:"full" size:"md"
    Chip variant:"solid" scheme:"primary" rounded:"full" size:"sm"
    Text font:"manrope"
    Title font:"manrope"
    theme name:"editorial"
      colors:
        primary color:"#315f4f" text:"#ffffff" title:"#ffffff"
        secondary color:"#171a18" text:"#ffffff" title:"#ffffff"
        tertiary color:"#8a7046" text:"#ffffff" title:"#ffffff"
        muted color:"#6c706a" text:"#ffffff" title:"#ffffff"
        background color:"#ecebe6" text:"#171a18" title:"#171a18"
        surface color:"#ffffff" text:"#171a18" title:"#171a18"
        success color:"#2f6b4f" text:"#ffffff" title:"#ffffff"
        info color:"#476579" text:"#ffffff" title:"#ffffff"
        warning color:"#8a682c" text:"#ffffff" title:"#ffffff"
        danger color:"#98504b" text:"#ffffff" title:"#ffffff"
"##;

const BLANK_ENV_EXAMPLE: &str = "BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=\n";
const BLANK_ENV: &str = "BACKEND_URL=http://127.0.0.1:8081\n";
const BLANK_ENV_LIVE: &str = "BACKEND_URL=\n";
const BLANK_ENV_STAGE: &str =
    "BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=replace-with-stage-password\n";
const BLANK_ENV_UAT: &str = "BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=replace-with-uat-password\n";

const BLANK_MAIN: &str = r#"import viewRoutes from "@/views/routes/view"
import apiRoutes from "@/server/endpoints"

main
  app name:"Hello Dowe" bundle:"dev.dowe.hello"
  views:viewRoutes
  server port:8081
    cors target:"server" devOrigins:true methods:["GET"] headers:["Content-Type"] credentials:false maxAge:600
    endpoints:apiRoutes
    databases:[]
"#;

const BLANK_VIEW_ROUTES: &str = r#"import homePage from "@/views/pages/home"

views viewRoutes
  route path:"/" page:homePage
"#;

const BLANK_API_ROUTES: &str = r#"import getHello from "@/server/handlers/hello"

endpoints apiRoutes
  get path:"/api/hello" handler:getHello
"#;

const BLANK_HOME_PAGE: &str = r#"page homePage
  Box bg:"background" color:"backgroundText" p:{ xs:6 md:10 }
    Grid columns:1 gap:5
      Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
        "DOWE"
      Title size:{ xs:"4xl" md:"6xl" } weight:"black"
        "Hello Dowe"
      Text size:"lg" color:"muted"
        "Your first page and Rust server endpoint are ready."
"#;

const BLANK_HELLO_HANDLER: &str = r#"handler getHello
  return text:"Hello Dowe"
"#;

const CRUD_ENV_EXAMPLE: &str = "BACKEND_URL=\nDOWE_HOST=\nDOWE_PORT=\nDOWE_USER=\nDOWE_PASSWORD=\nDOWE_DATABASE=\nCACHE_HOST=\nCACHE_PORT=\nCACHE_USER=\nCACHE_PASSWORD=\nCACHE_DATABASE=\nDOWE_DEPLOY_ACCESS_PASSWORD=\n";
const CRUD_ENV: &str = "BACKEND_URL=http://127.0.0.1:8081\nDOWE_HOST=127.0.0.1\nDOWE_PORT=4147\nDOWE_USER=local\nDOWE_PASSWORD=local\nDOWE_DATABASE=dowe-blog\nCACHE_HOST=127.0.0.1\nCACHE_PORT=4148\nCACHE_USER=local\nCACHE_PASSWORD=local\nCACHE_DATABASE=dowe-sessions\n";
const CRUD_ENV_LIVE: &str = "BACKEND_URL=\nDOWE_HOST=\nDOWE_PORT=\nDOWE_USER=\nDOWE_PASSWORD=\nDOWE_DATABASE=\nCACHE_HOST=\nCACHE_PORT=\nCACHE_USER=\nCACHE_PASSWORD=\nCACHE_DATABASE=\n";
const CRUD_ENV_STAGE: &str = "BACKEND_URL=\nDOWE_HOST=\nDOWE_PORT=\nDOWE_USER=\nDOWE_PASSWORD=\nDOWE_DATABASE=\nCACHE_HOST=\nCACHE_PORT=\nCACHE_USER=\nCACHE_PASSWORD=\nCACHE_DATABASE=\nDOWE_DEPLOY_ACCESS_PASSWORD=replace-with-stage-password\n";
const CRUD_ENV_UAT: &str = "BACKEND_URL=\nDOWE_HOST=\nDOWE_PORT=\nDOWE_USER=\nDOWE_PASSWORD=\nDOWE_DATABASE=\nCACHE_HOST=\nCACHE_PORT=\nCACHE_USER=\nCACHE_PASSWORD=\nCACHE_DATABASE=\nDOWE_DEPLOY_ACCESS_PASSWORD=replace-with-uat-password\n";

#[derive(Clone, Copy)]
struct InitTranslation {
    key: &'static str,
    en: &'static str,
    es: &'static str,
}

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

const CRUD_MAIN: &str = r#"import viewRoutes from "@/views/routes/view"
import apiRoutes from "@/server/endpoints"
import appDb from "@/server/config/database"

main
  app name:"Dowe Blogs" bundle:"dev.dowe.blogs"
  views:viewRoutes
  server port:8081
    cors target:"server" devOrigins:true methods:["GET" "POST" "PATCH"] headers:["Content-Type" "Authorization"] credentials:false maxAge:600
    endpoints:apiRoutes
    databases:[appDb]
"#;

const CRUD_VIEW_ROUTES: &str = r#"import AppLayout from "@/views/layouts/app"
import homePage from "@/views/pages/home"

views viewRoutes
  group path:"/" layout:AppLayout
    route path:"" page:homePage
"#;

const CRUD_LAYOUT: &str = r#"import session from "@/views/store/session"

layout AppLayout
  signal sessionLoading value:true
  init
    request res method:"GET" route:"/api/auth/session" headers:{ Authorization:session.authorization }
    if res.ok
      set session value:res.data
      set sessionLoading value:false
    else
      reset session
      set sessionLoading value:false
  Scaffold boxed:true
    appBar
      AppBar boxed:true variant:"ghost" scheme:"background" px:{ xs:4 md:8 } py:4
        start
          Flex align:"center" gap:3
            Avatar name:"Dowe Journal" alt:"Dowe Journal" variant:"solid" scheme:"primary" size:"md"
            Grid columns:1 gap:1
              Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
                "DOWE JOURNAL"
              Text size:"sm" color:"muted"
                "Editorial workspace"
        end
          Flex align:"center" gap:3
            Chip show:{ xs:false md:true } variant:"solid" scheme:"success" size:"sm"
              "RUST FULLSTACK"
            Avatar name:"Dowe Team" alt:"Dowe Team" variant:"solid" scheme:"secondary" size:"md" status:"online"
    main
      children
  Splash bind:sessionLoading
    Section minH:"vh-0" bg:"background" color:"backgroundText"
      Flex direction:"column" align:"center" justify:"center" gap:3 h:"full"
        Icon name:"svg-spinners:ring-resize" stroke:"primary" w:10 h:10
        Text size:"sm" color:"muted"
          "Validating your session"
"#;

const CRUD_SESSION_TYPE: &str = r#"type SessionUser
  id:string
  name:string
  email:string

type SessionState
  authenticated:bool
  guest:bool
  authorization:string
  token:string
  user:SessionUser
"#;

const CRUD_SESSION_STORE: &str = r#"import SessionState from "@/views/types/session"

store session:
  type:SessionState
  persistent:true
  value:{ authenticated:false guest:true authorization:"" token:"" user:{ id:"" name:"" email:"" } }
"#;

const CRUD_HOME_PAGE: &str = r#"import session from "@/views/store/session"

type BlogRow
  id:string
  title:string
  content:string
  ownerId:string
  createdAt:string

page homePage
  signal blogsLoading value:true
  signal blogs type:BlogRow[] value:[]
  signal registerForm value:{ name:"" email:"" password:"" }
  signal loginForm value:{ email:"" password:"" }
  signal blogForm value:{ title:"" content:"" }
  signal editForm value:{ id:"" title:"" content:"" }
  signal registerModalOpen value:false
  signal loginModalOpen value:false
  signal createModalOpen value:false
  signal editModalOpen value:false
  init
    request res method:"GET" route:"/api/blogs"
    if res.ok
      set blogs value:res.data
      set blogsLoading value:false
    else
      set blogsLoading value:false
      toast value:{ type:"error" title:"Error" message:"Could not load blogs." visible:true }
  fn openRegisterModal
    set registerModalOpen value:true
  fn closeRegisterModal
    set registerModalOpen value:false
  fn openLoginModal
    set loginModalOpen value:true
  fn closeLoginModal
    set loginModalOpen value:false
  fn openCreateModal
    set createModalOpen value:true
  fn closeCreateModal
    set createModalOpen value:false
  fn openEditModal
    set editModalOpen value:true
  fn closeEditModal
    set editModalOpen value:false
  fn loadBlogs
    set blogsLoading value:true
    request res method:"GET" route:"/api/blogs"
    if res.ok
      set blogs value:res.data
      set blogsLoading value:false
    else
      set blogsLoading value:false
      toast value:{ type:"error" title:"Error" message:"Could not load blogs." visible:true }
  fn register
    request res method:"POST" route:"/api/auth/register" body:registerForm
    if res.ok
      set session value:res.data
      set registerForm value:{ name:"" email:"" password:"" }
      set registerModalOpen value:false
      toast value:{ type:"success" title:"Account ready" message:"Welcome to the editorial workspace." visible:true }
    else
      toast value:{ type:"error" title:"Error" message:"Registration failed." visible:true }
  fn login
    request res method:"POST" route:"/api/auth/login" body:loginForm
    if res.ok
      set session value:res.data
      set loginForm value:{ email:"" password:"" }
      set loginModalOpen value:false
      toast value:{ type:"success" title:"Welcome back" message:"Your publishing tools are ready." visible:true }
    else
      toast value:{ type:"error" title:"Error" message:"Login failed." visible:true }
  fn createBlog
    request res method:"POST" route:"/api/blogs" body:blogForm headers:{ Authorization:session.authorization }
    if res.ok
      set blogs value:res.data
      set blogForm value:{ title:"" content:"" }
      set createModalOpen value:false
      toast value:{ type:"success" title:"Story published" message:"Your new story is now in the journal." visible:true }
    else
      toast value:{ type:"error" title:"Error" message:"Sign in before publishing." visible:true }
  fn updateBlog
    request res method:"PATCH" route:"/api/blogs/:id" body:editForm headers:{ Authorization:session.authorization }
    if res.ok
      set blogs value:res.data
      set editForm value:{ id:"" title:"" content:"" }
      set editModalOpen value:false
      toast value:{ type:"success" title:"Story updated" message:"Your changes were saved." visible:true }
    else
      toast value:{ type:"error" title:"Error" message:"Only the owner can edit that blog." visible:true }
  Section boxed:true px:{ xs:4 md:8 } py:{ xs:6 md:10 }
    Grid columns:1 gap:8
      Grid columns:{ xs:1 md:2 } gap:6 align:"end"
        Grid columns:1 gap:4
          Flex align:"center" gap:3 wrap:true
            Chip variant:"solid" scheme:"primary" size:"sm"
              "EDITORIAL WORKSPACE"
            Chip variant:"outlined" scheme:"success" size:"sm"
              "LIVE DEMO"
          Title size:{ xs:"4xl" md:"6xl" } weight:"black"
            "Stories worth returning to."
          Text size:"lg" color:"muted"
            "A polished fullstack journal with portable views, protected writing, and Rust-owned data."
          Flex gap:3 wrap:true
            Button show:session.guest onClick:openRegisterModal iconStart:"add-circle"
              "Create account"
            Button show:session.guest onClick:openLoginModal variant:"outlined" scheme:"secondary" iconStart:"user"
              "Sign in"
            Button show:session.authenticated onClick:openCreateModal iconStart:"add-circle"
              "New story"
            Button show:session.authenticated onClick:openEditModal variant:"outlined" scheme:"secondary" iconStart:"pen"
              "Edit my story"
        Card variant:"solid" scheme:"primary" p:5 rounded:"xl"
          Grid columns:1 gap:3
            Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
              "CURRENT SESSION"
            Title show:session.guest size:"xl" weight:"black"
              "Guest workspace"
            Title show:session.authenticated size:"xl" weight:"black"
              "{session.user.name}"
            Text show:session.guest size:"sm" color:"muted"
              "Sign in to publish and maintain your own stories."
            Text show:session.authenticated size:"sm" color:"muted"
              "Authenticated and ready to publish."
            Chip show:session.authenticated variant:"solid" scheme:"success" size:"sm"
              "OWNER VERIFIED"
  Section boxed:true px:{ xs:4 md:8 } pb:{ xs:8 md:12 }
    Grid columns:{ xs:1 md:2 } gap:5 align:"start"
      Card p:{ xs:4 md:6 } rounded:"xl"
        Grid columns:1 gap:5
          Flex justify:"between" align:"center" gap:4 wrap:true
            Grid columns:1 gap:1
              Text size:"xs" weight:"bold" spacing:"widest" color:"muted"
                "COMMUNITY JOURNAL"
              Title size:"2xl" weight:"black"
                "Latest stories"
            Button onClick:loadBlogs variant:"solid" scheme:"primary" size:"sm" iconStart:"restart"
              "Refresh"
          Table data:blogs variant:"ghost" scheme:"surface" size:"md" dividers:true emptyTitle:"The journal is ready" emptyDescription:"Create an account and publish the first story."
            column field:"title" label:"Story" width:"2fr"
            column field:"content" label:"Preview" width:"3fr"
            column field:"ownerId" label:"Writer" width:"1fr"
      Grid columns:1 gap:5
        Card p:5 rounded:"xl"
          Grid columns:1 gap:4
            Text size:"xs" weight:"bold" spacing:"widest" color:"muted"
              "QUICK ACTIONS"
            Title size:"xl" weight:"black"
              "Editorial controls"
            Text size:"sm" color:"muted"
              "Focused tasks open in their own workspace without crowding the journal."
            Button show:session.guest onClick:openRegisterModal iconStart:"add-circle"
              "Join the journal"
            Button show:session.guest onClick:openLoginModal variant:"solid" scheme:"secondary" iconStart:"user"
              "Access my account"
            Button show:session.authenticated onClick:openCreateModal iconStart:"add-circle"
              "Publish a story"
            Button show:session.authenticated onClick:openEditModal variant:"solid" scheme:"secondary" iconStart:"pen"
              "Revise a story"
        Card variant:"solid" scheme:"success" p:5 rounded:"xl"
          Grid columns:1 gap:3
            Avatar name:"Protected writing" alt:"Protected writing" variant:"solid" scheme:"success" size:"lg"
            Text size:"xs" weight:"bold" spacing:"widest" color:"success"
              "SECURE BY DESIGN"
            Title size:"xl" weight:"black"
              "Your stories stay yours."
            Text size:"sm" color:"muted"
              "The verified session subject is the only owner accepted by create and update."

  Modal open:registerModalOpen onClose:closeRegisterModal scheme:"surface"
    header
      Grid columns:1 gap:1
        Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
          "NEW WRITER"
        Title size:"2xl" weight:"black"
          "Create your account"
    Grid columns:1 gap:4
      Text size:"sm" color:"muted"
        "Join the journal to publish stories and maintain your own work."
      Input bind:registerForm.name label:"Full name" placeholder:"Ada Lovelace" labelFloating:true variant:"outlined" scheme:"primary" iconStart:"user" w:"full"
      Input bind:registerForm.email label:"Email" placeholder:"ada@example.com" labelFloating:true variant:"outlined" scheme:"primary" w:"full"
      Password bind:registerForm.password label:"Password" placeholder:"Create a password" labelFloating:true hideStrength:false variant:"outlined" scheme:"primary" w:"full"
    footer
      Flex justify:"end" gap:3 wrap:true
        Button onClick:closeRegisterModal variant:"ghost" scheme:"muted"
          "Cancel"
        Button onClick:register iconStart:"add-circle"
          "Create account"

  Modal open:loginModalOpen onClose:closeLoginModal scheme:"surface"
    header
      Grid columns:1 gap:1
        Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
          "WRITER ACCESS"
        Title size:"2xl" weight:"black"
          "Welcome back"
    Grid columns:1 gap:4
      Text size:"sm" color:"muted"
        "Continue writing with your protected editorial session."
      Input bind:loginForm.email label:"Email" placeholder:"ada@example.com" labelFloating:true variant:"outlined" scheme:"primary" w:"full"
      Password bind:loginForm.password label:"Password" placeholder:"Your password" labelFloating:true hideStrength:true variant:"outlined" scheme:"primary" w:"full"
    footer
      Flex justify:"end" gap:3 wrap:true
        Button onClick:closeLoginModal variant:"ghost" scheme:"muted"
          "Cancel"
        Button onClick:login iconStart:"user"
          "Sign in"

  Modal open:createModalOpen onClose:closeCreateModal scheme:"surface"
    header
      Grid columns:1 gap:1
        Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
          "NEW STORY"
        Title size:"2xl" weight:"black"
          "Publish something memorable"
    Grid columns:1 gap:4
      Text size:"sm" color:"muted"
        "Give the community a clear title and a thoughtful story."
      Input bind:blogForm.title label:"Story title" placeholder:"A new way to build" labelFloating:true variant:"outlined" scheme:"primary" iconStart:"pen" w:"full"
      Textarea bind:blogForm.content label:"Story" placeholder:"Write the story" rows:7 labelFloating:true variant:"outlined" scheme:"primary" w:"full"
    footer
      Flex justify:"end" gap:3 wrap:true
        Button onClick:closeCreateModal variant:"ghost" scheme:"muted"
          "Save for later"
        Button onClick:createBlog scheme:"success" iconStart:"check-circle"
          "Publish story"

  Modal open:editModalOpen onClose:closeEditModal scheme:"surface"
    header
      Grid columns:1 gap:1
        Text size:"xs" weight:"bold" spacing:"widest" color:"primary"
          "OWNER EDIT"
        Title size:"2xl" weight:"black"
          "Refine your story"
    Grid columns:1 gap:4
      Text size:"sm" color:"muted"
        "Use the story id from the journal. The backend verifies ownership before saving."
      Input bind:editForm.id label:"Story id" placeholder:"01J..." labelFloating:true variant:"outlined" scheme:"primary" w:"full"
      Input bind:editForm.title label:"Updated title" placeholder:"A sharper title" labelFloating:true variant:"outlined" scheme:"primary" iconStart:"pen" w:"full"
      Textarea bind:editForm.content label:"Updated story" placeholder:"Revise the story" rows:7 labelFloating:true variant:"outlined" scheme:"primary" w:"full"
    footer
      Flex justify:"end" gap:3 wrap:true
        Button onClick:closeEditModal variant:"ghost" scheme:"muted"
          "Cancel"
        Button onClick:updateBlog iconStart:"check-circle"
          "Save changes"
  Splash bind:blogsLoading
    Section minH:"vh-0" bg:"background" color:"backgroundText"
      Flex direction:"column" align:"center" justify:"center" gap:3 h:"full"
        Icon name:"svg-spinners:3-dots-bounce" fill:"primary" w:10 h:10
        Text size:"sm" color:"muted"
          "Loading the latest stories"
"#;

const BLANK_FILES: &[TemplateFile] = &[
    TemplateFile::new(".gitignore", GITIGNORE),
    TemplateFile::new(".zed/settings.json", ZED_SETTINGS),
    TemplateFile::new("theme.dowe", BLANK_THEME),
    TemplateFile::new(".env.example", BLANK_ENV_EXAMPLE),
    TemplateFile::new(".env", BLANK_ENV),
    TemplateFile::new(".env.live", BLANK_ENV_LIVE),
    TemplateFile::new(".env.stage", BLANK_ENV_STAGE),
    TemplateFile::new(".env.uat", BLANK_ENV_UAT),
    TemplateFile::new("main.dowe", BLANK_MAIN),
    TemplateFile::new("views/routes/view.dowe", BLANK_VIEW_ROUTES),
    TemplateFile::new("views/pages/home.dowe", BLANK_HOME_PAGE),
    TemplateFile::new("server/endpoints.dowe", BLANK_API_ROUTES),
    TemplateFile::new("server/handlers/hello.dowe", BLANK_HELLO_HANDLER),
];

const CRUD_FILES: &[TemplateFile] = &[
    TemplateFile::new(".gitignore", GITIGNORE),
    TemplateFile::new(".zed/settings.json", ZED_SETTINGS),
    TemplateFile::new("theme.dowe", CRUD_THEME),
    TemplateFile::new(".env.example", CRUD_ENV_EXAMPLE),
    TemplateFile::new(".env", CRUD_ENV),
    TemplateFile::new(".env.live", CRUD_ENV_LIVE),
    TemplateFile::new(".env.stage", CRUD_ENV_STAGE),
    TemplateFile::new(".env.uat", CRUD_ENV_UAT),
    TemplateFile::new("main.dowe", CRUD_MAIN),
    TemplateFile::new("views/routes/view.dowe", CRUD_VIEW_ROUTES),
    TemplateFile::new("views/layouts/app.dowe", CRUD_LAYOUT),
    TemplateFile::new("views/pages/home.dowe", CRUD_HOME_PAGE),
    TemplateFile::new("views/types/session.dowe", CRUD_SESSION_TYPE),
    TemplateFile::new("views/store/session.dowe", CRUD_SESSION_STORE),
    TemplateFile::new("server/endpoints.dowe", CRUD_API_ROUTES),
    TemplateFile::new("server/handlers/users-handler.dowe", CRUD_USERS_HANDLER),
    TemplateFile::new("server/handlers/blogs-handler.dowe", CRUD_BLOGS_HANDLER),
    TemplateFile::new("server/middlewares/auth.dowe", CRUD_AUTH_MIDDLEWARE),
    TemplateFile::new("server/config/database.dowe", CRUD_DATABASE),
    TemplateFile::new("server/entities/users-entity.dowe", CRUD_USERS),
    TemplateFile::new("server/entities/blogs-entity.dowe", CRUD_BLOGS),
    TemplateFile::new("server/entities/sessions-entity.dowe", CRUD_SESSIONS),
    TemplateFile::new("server/types/auth-types.dowe", CRUD_AUTH_TYPES),
    TemplateFile::new("server/types/blogs-types.dowe", CRUD_BLOG_TYPES),
    TemplateFile::new(
        "server/repositories/users-repository.dowe",
        CRUD_USERS_REPOSITORY,
    ),
    TemplateFile::new(
        "server/repositories/blogs-repository.dowe",
        CRUD_BLOGS_REPOSITORY,
    ),
    TemplateFile::new("server/services/users-service.dowe", CRUD_USERS_SERVICE),
    TemplateFile::new("server/services/blogs-service.dowe", CRUD_BLOGS_SERVICE),
];

pub(crate) fn files_for_options(options: InitProjectOptions) -> Vec<TemplateFile> {
    let (base, translations) = match options.template() {
        ProjectTemplate::Blank => (BLANK_FILES, BLANK_TRANSLATIONS),
        ProjectTemplate::Crud => (CRUD_FILES, CRUD_TRANSLATIONS),
    };
    files_with_translations(base.to_vec(), options, translations)
}

fn files_with_translations(
    mut files: Vec<TemplateFile>,
    options: InitProjectOptions,
    translations: &[InitTranslation],
) -> Vec<TemplateFile> {
    if options.i18n_enabled() {
        files = files
            .into_iter()
            .map(|file| localize_template_file(file, translations))
            .collect();
        files.push(TemplateFile::owned(
            "i18n/en.dowe",
            render_translation_catalog(translations, true, |entry| entry.en),
        ));
        files.push(TemplateFile::owned(
            "i18n/es.dowe",
            render_translation_catalog(translations, false, |entry| entry.es),
        ));
    }
    files
}

fn localize_template_file(file: TemplateFile, translations: &[InitTranslation]) -> TemplateFile {
    if !file.path().starts_with("views/") || !file.path().ends_with(".dowe") {
        return file;
    }
    TemplateFile::owned(
        file.path(),
        localize_view_source(file.content(), translations),
    )
}

fn localize_view_source(source: &str, translations: &[InitTranslation]) -> String {
    let mut lines = source.lines().map(str::to_owned).collect::<Vec<_>>();
    for index in 0..lines.len().saturating_sub(1) {
        let component = lines[index]
            .trim_start()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !matches!(component, "Text" | "Title" | "Button") {
            continue;
        }
        let fallback = lines[index + 1].trim();
        let Some(fallback) = fallback
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        let Some(translation) = translations.iter().find(|entry| entry.en == fallback) else {
            continue;
        };
        lines[index].push_str(&format!(" i18n:\"{}\"", translation.key));
    }
    let mut localized = lines.join("\n");
    if source.ends_with('\n') {
        localized.push('\n');
    }
    localized
}

fn render_translation_catalog(
    translations: &[InitTranslation],
    default: bool,
    value: impl Fn(&InitTranslation) -> &'static str,
) -> String {
    let mut catalog = if default {
        "translations default:true\n".to_string()
    } else {
        "translations\n".to_string()
    };
    let mut current_group = None;
    for translation in translations {
        let (group, leaf) = translation
            .key
            .split_once('.')
            .expect("init translation key group");
        assert!(!leaf.contains('.'), "init translation key depth");
        if current_group != Some(group) {
            catalog.push_str(&format!("  {group}\n"));
            current_group = Some(group);
        }
        catalog.push_str(&format!(
            "    {leaf} \"{}\"\n",
            escape_translation_value(value(translation))
        ));
    }
    catalog
}

fn escape_translation_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
