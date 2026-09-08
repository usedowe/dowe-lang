#[test]
fn binds_dowe_query_parameters_outside_literals() {
    assert_eq!(
        bind_query_params(
            "select * from users where name = ?1 and template = \"?2\"",
            &[json!("Ana"), json!("ignored")],
        )
        .expect("query"),
        "select * from users where name = \"Ana\" and template = \"?2\""
    );
}

#[test]
fn executes_documented_multi_join_query_with_portable_aliases() {
    let root = tempdir().expect("root");
    init_database(root.path(), "app").expect("database");
    let database = open_database(root.path(), "app").expect("open");
    for (table, value) in [
        (
            "users",
            json!({ "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "name": "Ana" }),
        ),
        (
            "roles",
            json!({ "id": "01ARZ3NDEKTSV4RRFFQ69G5FAW", "name": "admin" }),
        ),
        (
            "user_roles",
            json!({
                "id": "01ARZ3NDEKTSV4RRFFQ69G5FAX",
                "userId": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
                "roleId": "01ARZ3NDEKTSV4RRFFQ69G5FAW"
            }),
        ),
    ] {
        let Value::Object(record) = value else {
            unreachable!();
        };
        let record = record
            .into_iter()
            .map(|(field, value)| (field, StoreValue::from_json(value)))
            .collect::<StoreRecord>();
        database.insert(table, record).expect("insert");
    }
    let query = parse_select("SELECT users.name, roles.name AS roleName FROM users JOIN user_roles ON user_roles.userId = users.id JOIN roles ON user_roles.roleId = roles.id WHERE users.id = ?1").expect("query");
    let result = database
        .query_portable_json(&query, &[json!("01ARZ3NDEKTSV4RRFFQ69G5FAV")])
        .expect("result");
    assert_eq!(result, json!([{ "name": "Ana", "roleName": "admin" }]));
}

#[test]
fn executes_portable_identifier_filters() {
    let root = tempdir().expect("root");
    init_database(root.path(), "app").expect("database");
    let database = open_database(root.path(), "app").expect("open");
    let record = [
        (
            "id".to_string(),
            StoreValue::String("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
        ),
        (
            "externalId".to_string(),
            StoreValue::String("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
        ),
    ]
    .into_iter()
    .collect::<StoreRecord>();
    database.insert("users", record).expect("insert");
    let query = parse_select("SELECT id FROM users WHERE id = externalId").expect("query");

    let result = database.query_portable_json(&query, &[]).expect("result");

    assert_eq!(result, json!([{ "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV" }]));
}
