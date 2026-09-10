#[tokio::test]
async fn resolves_middleware_context_in_store_actions() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("middlewares")).expect("middlewares");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import requireBearer from "@/middlewares/auth"

main
  views:viewRoutes
  server port:0
    route "/api/blogs" middleware:[requireBearer]
      method POST async req
        const body value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query created conn:db.insert table:"blogs" value:{ title:body.title ownerId:req.context.auth.subject }
        return status:201 json:created
    route "/api/blogs/:id/edit" middleware:[requireBearer]
      method PATCH async req
        const body value:req.json
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"app"
        query updated conn:db.update table:"blogs" where:{ id:req.params.id ownerId:req.context.auth.subject } value:{ title:body.title } required:true
        return json:updated"#,
    )
    .expect("server");
    fs::write(
        temp.path().join("middlewares/auth.dowe"),
        r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let unauthorized = client
        .post(format!("{backend}/api/blogs"))
        .send()
        .await
        .expect("unauthorized");
    assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);

    let token = sign_jws_hs256(
        &json!({"sub":"blog-owner","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    let created = client
        .post(format!("{backend}/api/blogs"))
        .bearer_auth(token)
        .json(&json!({"title":"Owned post"}))
        .send()
        .await
        .expect("create");
    assert!(created.status().is_success());
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["ownerId"], "blog-owner");

    let other_token = sign_jws_hs256(
        &json!({"sub":"other-owner","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("other token");
    let denied = client
        .patch(format!("{backend}/api/blogs/{}/edit", created["id"]))
        .bearer_auth(other_token)
        .json(&json!({"title":"Attempted edit"}))
        .send()
        .await
        .expect("denied edit");
    assert_eq!(denied.status(), reqwest::StatusCode::NOT_FOUND);

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_layered_backend_with_middleware_functions_store_and_kv() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("layouts/auth.dowe"),
        r#"layout AuthLayout
  Box
    Text
      "Layout"
    children"#,
    )
    .expect("layout");
    fs::write(
        temp.path().join("pages/login.dowe"),
        r#"page loginPage
  Box
    Text
      "Login""#,
    )
    .expect("page");
    fs::create_dir_all(temp.path().join("handlers")).expect("handlers");
    fs::create_dir_all(temp.path().join("middlewares")).expect("middlewares");
    fs::create_dir_all(temp.path().join("server/services")).expect("services");
    fs::create_dir_all(temp.path().join("server/repositories")).expect("repositories");
    fs::create_dir_all(temp.path().join("types")).expect("types");
    fs::write(
        temp.path().join(".env"),
        "JWT_SECRET=01234567890123456789012345678901\n",
    )
    .expect("env");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import requireBearer from "@/middlewares/auth"
import listTickets from "@/handlers/tickets"
import createTicket from "@/handlers/tickets"

main
  views:viewRoutes
  server port:0
    route "/api/tickets" middleware:[requireBearer]
      method GET handler:listTickets
      method POST handler:createTicket"#,
    )
    .expect("main");
    fs::write(
        temp.path().join("middlewares/auth.dowe"),
        r#"middleware requireBearer params:{}
  bearer token value:req.header.Authorization
  jwt verified secret:env.JWT_SECRET algorithm:"HS256" token:token
  if verified.valid
    next context:{ auth:{ subject:verified.claims.sub } }
  return status:401 json:{ ok:false error:"Unauthorized" }"#,
    )
    .expect("middleware");
    fs::write(
        temp.path().join("types/tickets.dowe"),
        r#"type TicketInput
  title:string
  priority:string"#,
    )
    .expect("types");
    fs::write(
        temp.path().join("handlers/tickets.dowe"),
        r#"import TicketInput from "../types/tickets"
import listTicketsService from "@/server/services/tickets"
import createTicketService from "@/server/services/tickets"

handler listTickets req
  listTicketsService result args:{ status:"open" }
  return json:result

handler createTicket
  const body:TicketInput value:req.json
  createTicketService result args:{ title:body.title priority:body.priority status:"open" }
  return status:201 json:result"#,
    )
    .expect("handler");
    fs::write(
        temp.path().join("server/services/tickets.dowe"),
        r#"import listTicketsRepository from "@/server/repositories/tickets"
import createTicketRepository from "@/server/repositories/tickets"

fn listTicketsService params:{ status:string }
  listTicketsRepository result args:{ status:args.status }
  return value:{ ok:true data:result.rows cache:result.cache }

fn createTicketService params:{ title:string priority:string status:string }
  createTicketRepository result args:{ title:args.title priority:args.priority status:args.status }
  return value:{ ok:true data:result.rows created:result.created cache:result.cache }"#,
    )
    .expect("function");
    fs::write(
        temp.path().join("server/repositories/tickets.dowe"),
        r#"fn listTicketsRepository params:{ status:string }
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"support"
  query rows conn:db.list table:"tickets"
  cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"support-cache"
  kv saved conn:appCache.set key:"tickets:last-list" value:{ status:args.status }
  return value:{ rows:rows cache:saved }

fn createTicketRepository params:{ title:string priority:string status:string }
  database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"support"
  query created conn:db.insert table:"tickets" value:{ title:args.title priority:args.priority status:args.status createdAt:now updatedAt:now } required:["title","priority","status"]
  query rows conn:db.list table:"tickets"
  cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"support-cache"
  kv saved conn:appCache.set key:"tickets:last-created" value:{ id:created.id title:created.title }
  return value:{ rows:rows created:created cache:saved }"#,
    )
    .expect("function");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let unauthorized = client
        .get(format!("{backend}/api/tickets"))
        .send()
        .await
        .expect("unauthorized");
    assert_eq!(unauthorized.status(), reqwest::StatusCode::UNAUTHORIZED);

    let token = sign_jws_hs256(
        &json!({"sub":"agent-1","exp":4102444800u64}),
        "01234567890123456789012345678901",
    )
    .expect("token");
    let created = client
        .post(format!("{backend}/api/tickets"))
        .bearer_auth(&token)
        .json(&json!({"title":"Printer offline","priority":"high"}))
        .send()
        .await
        .expect("create");
    assert_eq!(created.status(), reqwest::StatusCode::CREATED);
    let created = created
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["ok"], true);
    assert_eq!(created["created"]["title"], "Printer offline");
    assert_eq!(created["cache"]["key"], "tickets:last-created");

    let listed = client
        .get(format!("{backend}/api/tickets"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("list")
        .json::<serde_json::Value>()
        .await
        .expect("list json");
    assert_eq!(listed["ok"], true);
    assert_eq!(listed["data"].as_array().expect("tickets").len(), 1);
    assert_eq!(listed["cache"]["key"], "tickets:last-list");
    assert!(temp.path().join(".dowe/db/support/tickets").exists());
    assert!(temp.path().join(".dowe/kv/support-cache").exists());

    servers.shutdown().await.expect("shutdown");
}

