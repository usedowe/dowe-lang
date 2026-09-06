#[test]
fn lowers_tree_with_file_hierarchy_binding_and_selection_action() {
    let tree = parse_page(
        r#"page treePage
  signal selected value:""
  signal files value:{
    folders:[
      { id:"src" name:"src" path:"src" type:"folder" files:[
        { id:"src/main.dowe" name:"main.dowe" path:"src/main.dowe" type:"file" }
      ] }
    ]
    files:[
      { id:"README.md" name:"README.md" path:"README.md" type:"file" }
    ]
  }
  fn selectFile
    set selected value:item.path
  Tree data:files bind:selected defaultOpen:false emptyLabel:"No files" ariaLabel:"Project files" onSelect:selectFile variant:"ghost" scheme:"surface""#,
    )
    .expect("tree");

    let ViewNode::Scope { children, actions, .. } = tree else {
        panic!("scope");
    };
    let ViewNode::Tree { props } = &children[0] else {
        panic!("tree node");
    };
    assert_eq!(props.data, "files");
    assert_eq!(props.bind.as_deref(), Some("selected"));
    assert!(!props.default_open);
    assert_eq!(props.empty_label, "No files");
    assert_eq!(props.aria_label, "Project files");
    assert_eq!(props.on_select.as_deref(), Some("selectFile"));
    assert_eq!(actions.len(), 1);
}

#[test]
fn validates_tree_generic_nodes_and_rejects_malformed_nodes() {
    let tree = parse_page(
        r#"page treePage
  signal navigation value:[
    {
      id:"guides"
      label:"Guides"
      kind:"branch"
      children:[
        { id:"guides/start" label:"Getting started" path:"guides/start" }
      ]
    }
  ]
  Tree data:navigation defaultOpen:true ariaLabel:"Guides""#,
    )
    .expect("generic tree");
    let ViewNode::Scope { children, .. } = tree else {
        panic!("scope");
    };
    assert!(matches!(children[0], ViewNode::Tree { .. }));

    let invalid = parse_page(
        r#"page treePage
  signal files value:{ files:[{ name:"main.dowe" }] folders:[] }
  Tree data:files"#,
    )
    .expect_err("invalid tree node");
    assert!(invalid.to_string().contains("Tree nodes must include a string or number `id`"));
}
