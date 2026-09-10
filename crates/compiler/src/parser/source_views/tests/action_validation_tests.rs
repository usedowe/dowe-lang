#[test]
fn rejects_inline_on_click_add_for_non_number() {
    let error = parse_page(
        r#"page menuPage
  signal label value:"Menu"
  Button onClick:{ set:label add:1 }
    "Open""#,
    )
    .expect_err("non-number inline target");

    assert!(
        error
            .message()
            .contains("invalid signal path `label` in `onClick add target`: expected number")
    );
}

#[test]
fn rejects_inline_on_click_append_for_non_string() {
    let error = parse_page(
        r#"page menuPage
  signal counter value:0
  Button onClick:{ set:counter append:"!" }
    "Open""#,
    )
    .expect_err("non-string inline target");

    assert!(
        error
            .message()
            .contains("invalid signal path `counter` in `onClick append target`: expected string")
    );
}

#[test]
fn rejects_assign_view_action() {
    let error = parse_page(
        r#"page menuPage
  signal openMenu value:false
  fn open
    assign openMenu source:true
  Box
    Text
      "Menu""#,
    )
    .expect_err("assign");

    assert!(
        error
            .message()
            .contains("`assign` was replaced by `set target value:<value>`")
    );
}

#[test]
fn rejects_negating_a_non_boolean_set_value() {
    let error = parse_page(
        r#"page menuPage
  signal count value:1
  fn toggle
    set count value:!count
  Box
    Text
      "Menu""#,
    )
    .expect_err("non-boolean set value");

    assert!(
        error
            .message()
            .contains("`set value:!count` must reference a boolean")
    );
}
