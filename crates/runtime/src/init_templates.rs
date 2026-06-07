use crate::init::{ProjectTemplate, TemplateFile};

const GITIGNORE: &str = ".dowe\n";

const BLANK_CONFIG: &str = r#"config
  fonts default:"inter" install:["inter"]
  env
    variable name:"BACKEND_URL" visibility:"client" required:false default:""
  server
    cors target:"server" devOrigins:true methods:["GET"] headers:["Content-Type"] credentials:false maxAge:600
"#;

const BLANK_MAIN: &str = r#"import getHello from "./handlers/hello"

main
  server port:8081
    route "/api/hello"
      method GET handler:getHello
    init
      log "Dowe server initialized"
"#;

const BLANK_VIEWS: &str = r#"import AppLayout from "./layouts/app"
import homePage from "./pages/home"

views
  route path:"/" layout:AppLayout
    page path:"" component:homePage
"#;

const BLANK_APP_LAYOUT: &str = r#"layout AppLayout
  Box bg:"background" color:"onBackground"
    AppBar variant:"soft" scheme:"surface" bordered:true boxed:true
      start
        Text weight:"bold"
          Dowe
      end
        Button href:"/" variant:"ghost" scheme:"primary" size:"sm"
          Home
    children
"#;

const BLANK_HOME_PAGE: &str = r#"type AlertState
  type:string
  message:string
  visible:bool

page homePage
  signal alert type:AlertState value:{ type:"info" message:"" visible:false }
  action load
    request GET route:"/api/hello" autoload:true
      onSuccess alert:"Hello handler answered"
      onError alert:"Hello handler failed"
  action close
    reset alert
  Box bg:"background" color:"onBackground"
    Grid columns:1 gap:6
      Card variant:"soft" scheme:"surface" p:{ xs:5 md:8 } rounded:"xl"
        Text size:"sm" weight:"semibold" color:"primary"
          DOWE PROJECT
        Title size:"5xl" weight:"bold"
          Hello from Dowe
        Text size:"lg" color:"muted"
          Your project is ready for server, web, desktop, Android, and iOS targets.
        Alert type:"info" message:alert.message visible:alert.visible onClose:close p:3
        Card variant:"outlined" scheme:"surface" p:5 rounded:"lg"
          Text size:"md"
            The /api/hello handler returns a text response from the Rust server target.
        Button onClick:load variant:"solid" scheme:"primary" size:"lg"
          Call hello handler
"#;

const BLANK_HELLO_HANDLER: &str = r#"handler getHello req
  return response text:"Hello from the Dowe server"
"#;

const BLANK_FILES: &[TemplateFile] = &[
    TemplateFile::new(".gitignore", GITIGNORE),
    TemplateFile::new("src/config.dowe", BLANK_CONFIG),
    TemplateFile::new("src/main.dowe", BLANK_MAIN),
    TemplateFile::new("src/views.dowe", BLANK_VIEWS),
    TemplateFile::new("src/layouts/app.dowe", BLANK_APP_LAYOUT),
    TemplateFile::new("src/pages/home.dowe", BLANK_HOME_PAGE),
    TemplateFile::new("src/handlers/hello.dowe", BLANK_HELLO_HANDLER),
];

const CLINIC_DESK_FILES: &[TemplateFile] = &[
    TemplateFile::new(".gitignore", GITIGNORE),
    TemplateFile::new(
        "src/config.dowe",
        include_str!("../../../examples/clinic-desk/src/config.dowe"),
    ),
    TemplateFile::new(
        "src/main.dowe",
        include_str!("../../../examples/clinic-desk/src/main.dowe"),
    ),
    TemplateFile::new(
        "src/views.dowe",
        include_str!("../../../examples/clinic-desk/src/views.dowe"),
    ),
    TemplateFile::new(
        "src/handlers/appointments.dowe",
        include_str!("../../../examples/clinic-desk/src/handlers/appointments.dowe"),
    ),
    TemplateFile::new(
        "src/layouts/workspace.dowe",
        include_str!("../../../examples/clinic-desk/src/layouts/workspace.dowe"),
    ),
    TemplateFile::new(
        "src/pages/appointments.dowe",
        include_str!("../../../examples/clinic-desk/src/pages/appointments.dowe"),
    ),
    TemplateFile::new(
        "src/pages/dashboard.dowe",
        include_str!("../../../examples/clinic-desk/src/pages/dashboard.dowe"),
    ),
];

const COMMERCE_OPS_FILES: &[TemplateFile] = &[
    TemplateFile::new(".gitignore", GITIGNORE),
    TemplateFile::new(
        "src/config.dowe",
        include_str!("../../../examples/commerce-ops/src/config.dowe"),
    ),
    TemplateFile::new(
        "src/main.dowe",
        include_str!("../../../examples/commerce-ops/src/main.dowe"),
    ),
    TemplateFile::new(
        "src/views.dowe",
        include_str!("../../../examples/commerce-ops/src/views.dowe"),
    ),
    TemplateFile::new(
        "src/handlers/products.dowe",
        include_str!("../../../examples/commerce-ops/src/handlers/products.dowe"),
    ),
    TemplateFile::new(
        "src/layouts/ops.dowe",
        include_str!("../../../examples/commerce-ops/src/layouts/ops.dowe"),
    ),
    TemplateFile::new(
        "src/pages/dashboard.dowe",
        include_str!("../../../examples/commerce-ops/src/pages/dashboard.dowe"),
    ),
    TemplateFile::new(
        "src/pages/inventory.dowe",
        include_str!("../../../examples/commerce-ops/src/pages/inventory.dowe"),
    ),
];

const SUPPORT_CONSOLE_FILES: &[TemplateFile] = &[
    TemplateFile::new(".gitignore", GITIGNORE),
    TemplateFile::new(
        "src/config.dowe",
        include_str!("../../../examples/support-console/src/config.dowe"),
    ),
    TemplateFile::new(
        "src/main.dowe",
        include_str!("../../../examples/support-console/src/main.dowe"),
    ),
    TemplateFile::new(
        "src/views.dowe",
        include_str!("../../../examples/support-console/src/views.dowe"),
    ),
    TemplateFile::new(
        "src/handlers/tickets.dowe",
        include_str!("../../../examples/support-console/src/handlers/tickets.dowe"),
    ),
    TemplateFile::new(
        "src/layouts/console.dowe",
        include_str!("../../../examples/support-console/src/layouts/console.dowe"),
    ),
    TemplateFile::new(
        "src/pages/dashboard.dowe",
        include_str!("../../../examples/support-console/src/pages/dashboard.dowe"),
    ),
    TemplateFile::new(
        "src/pages/tickets.dowe",
        include_str!("../../../examples/support-console/src/pages/tickets.dowe"),
    ),
];

pub(crate) fn files_for_template(template: ProjectTemplate) -> &'static [TemplateFile] {
    match template {
        ProjectTemplate::Blank => BLANK_FILES,
        ProjectTemplate::ClinicDesk => CLINIC_DESK_FILES,
        ProjectTemplate::CommerceOps => COMMERCE_OPS_FILES,
        ProjectTemplate::SupportConsole => SUPPORT_CONSOLE_FILES,
    }
}
