#[test]
fn rejects_non_type_block_inside_shared_type_module() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("types")).expect("types");
    let path = root.join("types/tickets.dowe");
    let file = parse_source_file(
        root,
        &path,
        r#"type Ticket
  id:string

handler listTickets req
  return json:{ ok:true }"#
            .to_string(),
    )
    .expect("source");

    let error = validate_shared_type_source(root, &file).expect_err("type module error");

    assert!(
        error
            .to_string()
            .contains("shared type modules only accept `type`")
    );
}

#[test]
fn rejects_shared_type_import_cycles() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("types")).expect("types");
    fs::write(
        root.join("types/a.dowe"),
        r#"import B from "@/types/b"

type A
  b:B"#,
    )
    .expect("type a");
    fs::write(
        root.join("types/b.dowe"),
        r#"import A from "@/types/a"

type B
  a:A"#,
    )
    .expect("type b");
    let path = root.join("types/a.dowe");
    let source = fs::read_to_string(&path).expect("source");
    let file = parse_source_file(root, &path, source).expect("file");

    let error = TypeRegistry::parse_file(root, &file).expect_err("cycle");

    assert!(
        error
            .to_string()
            .contains("cyclic shared type import detected")
    );
}

#[test]
fn resolves_shared_types_from_arbitrary_module_paths() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("domains/tickets/contracts")).expect("contracts");
    fs::write(
        root.join("domains/tickets/contracts/summary.dowe"),
        "type TicketSummary\n  id:string\n  title:string\n",
    )
    .expect("type source");
    let path = root.join("screens/tickets.dowe");
    fs::create_dir_all(path.parent().expect("parent")).expect("screens");
    let file = parse_source_file(
            root,
            &path,
            "import TicketSummary from \"../domains/tickets/contracts/summary\"\n\npage ticketsPage\n  signal tickets type:TicketSummary[] value:[]\n  Text\n    \"Tickets\"\n"
                .to_string(),
        )
        .expect("source");

    TypeRegistry::parse_file(root, &file).expect("type module should be classified by declaration");
}
