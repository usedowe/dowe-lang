use super::{
    compile_dev, compile_for_environment, compile_for_server_environment,
    compile_for_web_environment,
};
use crate::model::{
    CompileEnvironment, EndpointBehavior, EnvironmentValueSource, EnvironmentVisibility,
    HttpMethod, ServerLogLevel, ServerLogValue, ServerStatement,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn generated_css_chunk<'a>(paths: &'a [String], prefix: &str) -> &'a str {
    paths
        .iter()
        .find(|path| path.starts_with(prefix))
        .map(String::as_str)
        .expect("generated css chunk")
}

fn android_dev_output(root: &Path) -> String {
    let source_root = root.join(".dowe/apps/android/dev/src/dev/dowe/generated");
    let core = fs::read_to_string(source_root.join("DoweDevActivity.java"))
        .expect("android dev activity");
    let mut output = core
        .lines()
        .map(|line| {
            if let Some(declaration) = line.strip_prefix("    ")
                && !declaration.starts_with(' ')
            {
                format!("    private {declaration}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    output.push('\n');
    let mut shards = fs::read_dir(&source_root)
        .expect("android dev sources")
        .map(|entry| entry.expect("android dev source").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    (name.starts_with("DoweDevRoute") || name.starts_with("DoweDevLayout"))
                        && name.ends_with(".java")
                })
        })
        .collect::<Vec<_>>();
    shards.sort();
    for path in shards {
        output.push_str(
            &fs::read_to_string(path)
                .expect("android dev source")
                .replace(
                    "int viewportWidth = runtime.viewportWidth;",
                    "int viewportWidth = this.viewportWidth;",
                )
                .replace("runtime.", "")
                .replace("runtime", "this")
                .replace("DoweDevActivity.", ""),
        );
        output.push('\n');
    }
    output
}

fn write_blog_fixture(root: &Path) {
    write_fixture_with_views(
        root,
        r#"layout AuthLayout
  signal alert value:{ type:"info" message:"Layout alert" visible:true }
  fn close
    reset alert
  Box
    Text
      "Layout"
    Alert type:"info" message:alert.message visible:alert.visible onClose:close
    children"#,
        r#"page loginPage
  Box
    Text
      "Login""#,
    );
    fs::write(
        root.join("theme.dowe"),
        r#"theme
  fonts default:"inter" install:["inter"]"#,
    )
    .expect("theme");
    fs::write(root.join(".env.example"), "BACKEND_URL=\nINTERNAL_TOKEN=\n")
        .expect("env example");
    fs::write(root.join(".env"), "BACKEND_URL=\nINTERNAL_TOKEN=\n").expect("env");
    fs::create_dir_all(root.join("handlers")).expect("handlers");
    fs::write(
        root.join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import listBlogs from "@/handlers/blogs"
import createBlog from "@/handlers/blogs"
import readBlog from "@/handlers/blogs"
import updateBlog from "@/handlers/blogs"
import deleteBlog from "@/handlers/blogs"

main
  views:viewRoutes
  server port:8080
    route "/api/blogs"
      method GET handler:listBlogs
      method POST handler:createBlog
    route "/api/blogs/:id"
      method GET handler:readBlog
      method PATCH handler:updateBlog
      method DELETE handler:deleteBlog"#,
    )
    .expect("server");
    fs::write(
            root.join("handlers/blogs.dowe"),
        r#"handler listBlogs req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler createBlog
  const body value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query created conn:db.insert table:"blogs" value:{ title:body.title content:body.content createdAt:now updatedAt:now } required:["title","content"]
  query blogs conn:db.list table:"blogs"
  return status:201 json:{ ok:true data:blogs }

handler readBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query blog conn:db.read table:"blogs" where:{ id:req.params.id } required:true
  return json:{ ok:true data:blog }

handler updateBlog
  const body value:req.json
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query updated conn:db.update table:"blogs" where:{ id:req.params.id } value:{ title:body.title content:body.content updatedAt:now } required:true match:{ id:req.params.id }
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }

handler deleteBlog req
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
  query deleted conn:db.delete table:"blogs" where:{ id:req.params.id } required:true
  query blogs conn:db.list table:"blogs"
  return json:{ ok:true data:blogs }"#,
        )
        .expect("handlers");
    fs::write(
        root.join("routes/view.dowe"),
        r#"import AuthLayout from "../layouts/auth"
import loginPage from "../pages/login"
import blogsPage from "../pages/blogs"

views viewRoutes
  group path:"/" layout:AuthLayout
    route path:"" page:loginPage
    route path:"blogs" page:blogsPage"#,
    )
    .expect("views");
    fs::write(
        root.join("pages/blogs.dowe"),
        r#"page blogsPage
  signal blog value:{ id:null title:"" content:"" }
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  fn load
    request GET route:"/api/blogs" update:blogs autoload:true
      onError alert:"No se pudieron cargar los blogs"
  fn create
    request POST route:"/api/blogs" body:blog update:blogs reset:blog
      onSuccess alert:"Blog creado"
      onError alert:"No se pudo crear el blog"
  fn edit
    set blog value:item
  fn update
    request PATCH route:"/api/blogs/:id" body:blog update:blogs reset:blog
      onSuccess alert:"Blog actualizado"
      onError alert:"No se pudo actualizar el blog"
  fn delete
    request DELETE route:"/api/blogs/:id" body:item update:blogs
      onSuccess alert:"Blog eliminado"
      onError alert:"No se pudo eliminar el blog"
  fn close
    reset alert
  Box
    Title
      "Blogs"
    Alert type:"info" message:alert.message visible:alert.visible onClose:close
    Input bind:blog.title
    Button onClick:create
      "Crear"
    each in:blogs as:item key:item.id
      Card
        Title
          "{item.title}"
        Text
          "{item.content}"
        Text
          "item.literal"
        Button onClick:edit
          "Editar""#,
    )
    .expect("blogs");
}

fn attribute_values<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let prefix = format!(r#"{name}=""#);
    html.match_indices(&prefix)
        .filter_map(|(start, _)| {
            let value = &html[start + prefix.len()..];
            value.find('"').map(|end| &value[..end])
        })
        .collect()
}

fn short_root(value: &str, suffix: &str) -> bool {
    value.strip_suffix(suffix).is_some_and(|root| {
        root.len() == 8
            && root
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    })
}

fn ios_swift_output(root: &Path) -> String {
    ios_swift_output_from(&root.join(".dowe/apps/ios"))
}

fn ios_apps_swift_output(root: &Path) -> String {
    ios_swift_output_from(&root.join("ios"))
}

fn ios_swift_output_from(ios_root: &Path) -> String {
    let mut swift_files = fs::read_dir(ios_root)
        .expect("ios output")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("swift"))
        .collect::<Vec<_>>();
    swift_files.sort();
    swift_files
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("ios swift"))
        .collect::<Vec<_>>()
        .join("\n")
}

include!("tests_config/project.rs");
include!("tests_config/platform.rs");
include!("tests_config/validation.rs");
include!("tests_config/app_metadata.rs");
include!("tests_config/views_and_targets.rs");
include!("tests_config/environment.rs");
include!("tests_config/cors.rs");
