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

  Modal bind:registerModalOpen onClose:closeRegisterModal scheme:"surface"
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

  Modal bind:loginModalOpen onClose:closeLoginModal scheme:"surface"
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

  Modal bind:createModalOpen onClose:closeCreateModal scheme:"surface"
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

  Modal bind:editModalOpen onClose:closeEditModal scheme:"surface"
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
    let files = base
        .iter()
        .cloned()
        .map(|file| customize_main_file(file, &options))
        .collect();
    files_with_translations(files, options, translations)
}


