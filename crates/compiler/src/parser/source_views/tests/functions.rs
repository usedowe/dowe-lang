    #[test]
    fn parses_native_invoke_statement() {
        let page = parse_page(
            r#"page settingsPage
  signal settings value:null
  fn load
    invoke result fn:readSettings args:{ userId:"abc" } update:settings
  Text
    "Settings""#,
        )
        .expect("page");
        let ViewNode::Scope { actions, .. } = page else {
            panic!("scope");
        };
        let ViewActionKind::Sequence(statements) = &actions[0].kind else {
            panic!("sequence");
        };
        assert!(matches!(
            &statements[0],
            ViewFunctionStatement::Invoke { result, action }
                if result == "result" && action.function == "readSettings" && action.args.len() == 1
        ));
    }

    #[test]
    fn rejects_incompatible_view_function_parameter() {
        let error = parse_page(
            r#"type Appointment
  startsAt:string

page appointmentsPage
  signal appointment value:""
  fn create params:{ appointment:Appointment }
    request POST route:"/api/appointments" body:appointment
  Text
    "Appointments""#,
        )
        .expect_err("parameter type");

        assert!(
            error
                .to_string()
                .contains("fn parameter `appointment` does not match declared type `Appointment`")
        );
    }

    fn parse_page(source: &str) -> crate::error::DoweResult<ViewNode> {
        let root = Path::new("/project");
        let path = Path::new("/project/pages/blogs.dowe");
        let file = parse_source_file(root, path, source.to_string())?;
        validate_view_source(root, &file, &environment())
    }
