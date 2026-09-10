#[test]
fn validates_variant_props() {
    let node = input_node(vec![
        string_prop("variant", "ghost"),
        string_prop("scheme", "danger"),
        string_prop("bind", "blog.title"),
        string_prop("label", "Title"),
        string_prop("placeholder", "Write a title"),
        boolean_prop("labelFloating", true),
    ])
    .expect("input");

    match node {
        ViewNode::Input { props } => {
            assert_eq!(props.variant, Some(ComponentVariant::Ghost));
            assert_eq!(props.color, Some(ColorFamily::Danger));
            assert_eq!(props.element.bind.as_deref(), Some("blog.title"));
            assert_eq!(props.label.as_deref(), Some("Title"));
            assert_eq!(props.placeholder.as_deref(), Some("Write a title"));
            assert!(props.label_floating);
        }
        _ => panic!("input"),
    }
}

