    #[test]
    fn rejects_each_over_non_array_signal() {
        let error = parse_page(
            r#"page blogsPage
  signal blog value:{ id:"" title:"" }
  each in:blog as:item key:item.id
    Text
      "{item.title}""#,
        )
        .expect_err("collection type");

        assert!(error.to_string().contains("must be an array"));
    }

    #[test]
    fn validates_typed_signals_and_empty_typed_collections() {
        parse_page(
            r#"type BlogForm
  id?:string
  title:string
  content:string

type BlogItem
  id:string
  title:string

page blogsPage
  signal blog type:BlogForm value:{ id:null title:"" content:"" }
  signal blogs type:BlogItem[] value:[]
  Box
    Input bind:blog.title
    each in:blogs as:item key:item.id
      Text
        "{item.title}""#,
        )
        .expect("typed page");

        let error = parse_page(
            r#"type BlogItem
  id:string
  title:string

page blogsPage
  signal blogs type:BlogItem[] value:[]
  Box
    each in:blogs as:item key:item.id
      Text
        "{item.missing}""#,
        )
        .expect_err("missing typed field");

        assert!(
            error
                .to_string()
                .contains("unknown signal path `item.missing`")
        );

        parse_page(
            r#"page literalPage
  signal blogs value:[{ id:"one" title:"Title" }]
  each in:blogs as:item key:item.id
    Text
      "item.missing""#,
        )
        .expect("dotted literal text");

        parse_page(
            r#"page malformedPage
  signal blogs value:[{ id:"one" title:"Title" }]
  each in:blogs as:item key:item.id
    Text
      "Hello {item.title}""#,
        )
        .expect("mixed text remains literal");

        let non_string = parse_page(
            r#"page nonStringPage
  signal blogs value:[{ id:"one" count:1 }]
  each in:blogs as:item key:item.id
    Text
      "{item.count}""#,
        )
        .expect_err("non-string text binding");

        assert!(
            non_string
                .to_string()
                .contains("invalid signal path `item.count` in `text`: expected string")
        );
    }

    #[test]
    fn validates_shared_type_imported_by_view_signal() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("types")).expect("types");
        fs::create_dir_all(root.join("pages")).expect("pages");
        fs::write(
            root.join("types/tickets.dowe"),
            r#"type TicketSummary
  id:string
  title:string
  status:string"#,
        )
        .expect("type source");
        let path = root.join("pages/tickets.dowe");
        let source = r#"import TicketSummary from "../types/tickets"

page ticketsPage
  signal tickets type:TicketSummary[] value:[]
  Box
    each in:tickets as:item key:item.id
      Text
        "{item.title}""#
            .to_string();
        let file = parse_source_file(root, &path, source).expect("source");

        validate_view_source(root, &file, &environment()).expect("view");
    }

    #[test]
    fn imports_persistent_view_store_from_arbitrary_module_path() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("domains/auth/contracts")).expect("types");
        fs::create_dir_all(root.join("domains/auth/state")).expect("store");
        fs::create_dir_all(root.join("views/pages")).expect("pages");
        fs::write(
            root.join("domains/auth/contracts/session.dowe"),
            r#"type SessionState
  authorization:string
  user:SessionUser

type SessionUser
  id:string
  name:string
  email:string"#,
        )
        .expect("type source");
        fs::write(
            root.join("domains/auth/state/session.dowe"),
            r#"import SessionState from "../contracts/session"

store session type:SessionState persistent:true value:{ authorization:"" user:{ id:"" name:"" email:"" } }"#,
        )
        .expect("store source");
        let path = root.join("views/pages/auth.dowe");
        let source = r#"import session from "@/domains/auth/state/session"

page authPage
  signal loginForm value:{ email:"" password:"" }
  fn clearAuthorization
    set session.authorization value:""
  fn login
    request POST route:"/api/auth/login" body:loginForm update:session
  Text
    "Authentication""#
            .to_string();
        let file = parse_source_file(root, &path, source).expect("source");

        let tree = validate_view_source(root, &file, &environment()).expect("view");
        let ViewNode::Scope {
            signals, actions, ..
        } = tree
        else {
            panic!("scope");
        };
        let store = signals
            .iter()
            .find(|signal| signal.name == "session")
            .expect("session store");
        assert_eq!(store.scope, ViewSignalScope::Global);
        assert_eq!(store.storage, ViewSignalStorage::Local);
        assert_eq!(store.storage_key, "domains/auth/state/session:session");
        let ViewActionKind::Assign(action) = &actions[0].kind else {
            panic!("set");
        };
        assert_eq!(action.target, "session.authorization");
        assert_eq!(action.source, "$dowe:string:");
    }

    #[test]
    fn lowers_inline_on_click_for_imported_view_store_path() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("views/store")).expect("store");
        fs::create_dir_all(root.join("views/pages")).expect("pages");
        fs::write(
            root.join("views/store/preferences.dowe"),
            r#"store preferences persistent:true value:{ compactNavigation:false }"#,
        )
        .expect("store source");
        let path = root.join("views/pages/menu.dowe");
        let source = r#"import preferences from "../store/preferences"

page menuPage
  Button onClick:{ set:preferences.compactNavigation value:!preferences.compactNavigation }
    "Compact navigation""#
            .to_string();
        let file = parse_source_file(root, &path, source).expect("source");

        let tree = validate_view_source(root, &file, &environment()).expect("view");
        let ViewNode::Scope { actions, .. } = tree else {
            panic!("scope");
        };
        let ViewActionKind::Assign(action) = &actions[0].kind else {
            panic!("inline set action");
        };
        assert_eq!(action.target, "preferences.compactNavigation");
        assert_eq!(action.source, "!preferences.compactNavigation");
    }

    #[test]
    fn rejects_invalid_view_store_module_shape() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("views/store")).expect("store");
        fs::create_dir_all(root.join("views/pages")).expect("pages");
        fs::write(
            root.join("views/store/session.dowe"),
            r#"store session persistent:"true" value:{ authorization:"" }"#,
        )
        .expect("store source");
        let path = root.join("views/pages/auth.dowe");
        let source = r#"import session from "../store/session"

page authPage
  Text
    session.authorization"#
            .to_string();
        let file = parse_source_file(root, &path, source).expect("source");

        let error = validate_view_source(root, &file, &environment()).expect_err("store error");
        assert!(
            error
                .to_string()
                .contains("`store persistent` must be a boolean")
        );
    }

    #[test]
    fn accepts_view_store_under_any_project_folder() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("local")).expect("local");
        let path = root.join("local/session.dowe");
        let file = parse_source_file(
            root,
            &path,
            r#"store session value:{ authorization:"" }"#.to_string(),
        )
        .expect("source");

        validate_view_store_source(root, &file).expect("declaration-based store module");
    }

    #[test]
    fn rejects_result_block_with_inline_result_props() {
        let error = parse_page(
            r#"page blogsPage
  signal blogs value:[]
  signal alert value:{ type:"info" message:"" visible:false }
  fn create
    request POST route:"/api/blogs" update:blogs successAlert:alert
      onSuccess alert:"Blog creado"
  Box
    Text
      "Blogs""#,
        )
        .expect_err("error");

        assert!(
            error
                .to_string()
                .contains("cannot be combined with inline success props")
        );
    }
