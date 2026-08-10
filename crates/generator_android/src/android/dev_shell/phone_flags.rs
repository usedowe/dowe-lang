const DEV_PHONE_FLAGS_PER_SHARD: usize = 24;

fn dev_tree_has_phone(node: &ViewNode) -> bool {
    matches!(node, ViewNode::Phone { .. })
        || node_child_groups(node)
            .into_iter()
            .flatten()
            .any(dev_tree_has_phone)
}

fn dev_activity_phone_flag_runtime() -> &'static str {
    r#"    private DoweSvgView dowePhoneFlag(String code, int color) {
        return DoweDevPhoneFlags.create(this, code, color);
    }

    private String[] dowePhoneCodes() {
        return DoweDevPhoneFlags.CODES;
    }

    private String[] dowePhoneNames() {
        return DoweDevPhoneFlags.NAMES;
    }

    private String[] dowePhoneDials() {
        return DoweDevPhoneFlags.DIALS;
    }

"#
}

fn dev_activity_empty_phone_flag_runtime() -> &'static str {
    r#"    private DoweSvgView dowePhoneFlag(String code, int color) {
        return null;
    }

"#
}

fn dev_phone_flag_shards(app_bundle: &str) -> Vec<DevActivityShard> {
    let flags = phone_countries()
        .iter()
        .filter_map(|country| {
            phone_country_flag_icon(country.code).map(|icon| (country.code, icon))
        })
        .collect::<Vec<_>>();
    let mut shards = vec![dev_phone_flag_dispatcher(&flags, app_bundle)];
    shards.extend(
        flags
            .chunks(DEV_PHONE_FLAGS_PER_SHARD)
            .enumerate()
            .map(|(index, flags)| dev_phone_flag_group(index, flags, app_bundle)),
    );
    shards
}

fn dev_phone_flag_dispatcher(flags: &[(&str, SideNavIcon)], app_bundle: &str) -> DevActivityShard {
    let mut output = dev_shard_header(app_bundle);
    output.push_str(
        "@SuppressWarnings({\"unchecked\", \"deprecation\"})\nfinal class DoweDevPhoneFlags {\n",
    );
    output.push_str(&format!(
        "    static final String[] CODES = {};\n    static final String[] NAMES = {};\n    static final String[] DIALS = {};\n\n",
        java_string_array(phone_countries().iter().map(|country| country.code)),
        java_string_array(phone_countries().iter().map(|country| country.name)),
        java_string_array(phone_countries().iter().map(|country| country.dial)),
    ));
    output.push_str("    private DoweDevPhoneFlags() {}\n\n    static DoweDevActivity.DoweSvgView create(DoweDevActivity runtime, String code, int color) {\n        if (code == null) {\n");
    if let Some((code, _)) = flags.first() {
        output.push_str(&format!(
            "            return DoweDevPhoneFlags0.flag{code}(runtime, color);\n"
        ));
    } else {
        output.push_str("            return null;\n");
    }
    output.push_str("        }\n        switch (code) {\n");
    for (position, (code, _)) in flags.iter().enumerate() {
        let shard = position / DEV_PHONE_FLAGS_PER_SHARD;
        output.push_str(&format!(
            "            case \"{code}\": return DoweDevPhoneFlags{shard}.flag{code}(runtime, color);\n"
        ));
    }
    if let Some((code, _)) = flags.first() {
        output.push_str(&format!(
            "            default: return DoweDevPhoneFlags0.flag{code}(runtime, color);\n"
        ));
    } else {
        output.push_str("            default: return null;\n");
    }
    output.push_str("        }\n    }\n}\n");
    DevActivityShard {
        file_name: "DoweDevPhoneFlags.java".to_string(),
        content: output,
    }
}

fn dev_phone_flag_group(
    index: usize,
    flags: &[(&str, SideNavIcon)],
    app_bundle: &str,
) -> DevActivityShard {
    let class_name = format!("DoweDevPhoneFlags{index}");
    let mut output = dev_shard_header(app_bundle);
    output.push_str(&format!(
        "@SuppressWarnings({{\"unchecked\", \"deprecation\"}})\nfinal class {class_name} {{\n    private {class_name}() {{}}\n\n"
    ));
    for (code, icon) in flags {
        output.push_str(&format!(
            "    static DoweDevActivity.DoweSvgView flag{code}(DoweDevActivity runtime, int color) {{\n        int viewportWidth = runtime.viewportWidth;\n"
        ));
        let mut body = String::new();
        let mut counter = 0;
        let view = render_dev_android_icon_view(icon, &mut counter, &mut body, Some("color"));
        output.push_str(&qualify_dev_shard_fragment(&body));
        output.push_str(&format!("        return {view};\n    }}\n\n"));
    }
    output.push_str("}\n");
    DevActivityShard {
        file_name: format!("{class_name}.java"),
        content: output,
    }
}
