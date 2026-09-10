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
        primary color:"#1F3A5F" text:"#EBF2FA" title:"#FFFFFF"
        secondary color:"#6BC670" text:"#0F291E" title:"#040D05"
        accent color:"#3F7A8A" text:"#F0F7F9" title:"#FFFFFF"
        muted color:"#E2E8F0" text:"#334155" title:"#1F3A5F"
        background color:"#F3F1EE" text:"#334155" title:"#1F3A5F"
        surface color:"#FFFFFF" text:"#334155" title:"#1F3A5F"
        success color:"#16A34A" text:"#E8F5E9" title:"#FFFFFF"
        info color:"#0084D1" text:"#E1F5FE" title:"#FFFFFF"
        warning color:"#D08700" text:"#1F1400" title:"#0D0900"
        danger color:"#E7000B" text:"#FFEBEE" title:"#FFFFFF"
    theme name:"dark" extends:"light"
      colors:
        primary color:"#F3F1EE" text:"#334155" title:"#1F3A5F"
        muted color:"#334155" text:"#D5DEE9" title:"#F3F7FC"
        background color:"#111827" text:"#E5E7EB" title:"#F9FAFB"
        surface color:"#1F2937" text:"#E5E7EB" title:"#F9FAFB"
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
        accent color:"#8a7046" text:"#ffffff" title:"#ffffff"
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


