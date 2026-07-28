    fn openrouter_environment(visibility: EnvironmentVisibility) -> EnvironmentConfig {
        EnvironmentConfig {
            variables: vec![
                EnvironmentVariable {
                    name: "OPENROUTER_BASE_URL".to_string(),
                    visibility: EnvironmentVisibility::Server,
                    resolved_source: EnvironmentValueSource::Missing,
                    resolved_value: None,
                },
                EnvironmentVariable {
                    name: "OPENROUTER_API_KEY".to_string(),
                    visibility,
                    resolved_source: EnvironmentValueSource::Missing,
                    resolved_value: None,
                },
            ],
        }
    }

    fn write_session_middleware_project(root: &Path, verification: &str) {
        fs::create_dir_all(root.join("server/config")).expect("config");
        fs::create_dir_all(root.join("server/middlewares")).expect("middlewares");
        fs::write(
            root.join("main.dowe"),
            r#"import requireBearer from "@/server/middlewares/auth"

main
  server port:8080
    route "/api/private" middleware:[requireBearer]
      handler
        return text:"Private""#,
        )
        .expect("main");
        fs::write(
            root.join("server/config/database.dowe"),
            r#"database appDb provider:"dowe" host:"127.0.0.1" port:4147 account:"app" secret:"secret" name:"app" entities:[] seeders:[]
cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"app""#,
        )
        .expect("config");
        fs::write(
            root.join("server/middlewares/auth.dowe"),
            format!(
                r#"import {{ appDb, appCache }} from "@/server/config/database"

middleware requireBearer
  bearer token value:req.header.Authorization
  {verification}
  if verified.valid
    next context:{{ auth:{{ subject:verified.userId session:verified.id }} }}
  return status:401 json:{{ ok:false error:"Unauthorized" }}"#
            ),
        )
        .expect("middleware");
    }
