#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_nested_svg_paths_to_dowe_source() {
        let source = r#"<?xml version="1.0"?><svg width="48px" height="24px"><g transform="matrix(2,0,0,2,4,6)"><path d="M0 0L8 0Z" style="fill:rgb(31,58,95)"/><path d="M0 1L8 1Z" fill="rgb(107,198,112)"/></g></svg>"#;
        let output = convert_svg(source, false).expect("svg");

        assert!(output.starts_with("Svg viewBox:\"0 0 48 24\" w:\"full\" h:\"full\""));
        assert!(output.contains("fill:\"primary\" transform:\"matrix(2 0 0 2 4 6)\""));
        assert!(output.contains("fill:\"secondary\" transform:\"matrix(2 0 0 2 4 6)\""));
    }

    #[test]
    fn ignores_external_doctype_without_resolving_it() {
        let source = r#"<?xml version="1.0"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="627px" height="145px"><g transform="matrix(1,0,0,1,-983.055297,-2551.972932)"><path d="M0 0L1 1Z" style="fill:rgb(31,58,95)"/></g></svg>"#;

        let output = convert_svg(source, false).expect("svg source");

        assert!(output.starts_with("Svg viewBox:\"0 0 627 145\""));
        assert!(output.contains("fill:\"primary\""));
        assert!(output.contains("transform:\"matrix(1 0 0 1 -983.055297 -2551.972932)\""));
    }

    #[test]
    fn converts_rectangles_and_coalesces_near_rgb_fills() {
        let source = r#"<svg viewBox="0 0 40 20"><path d="M0 0L1 1Z" fill="rgb(5,5,3)"/><g transform="matrix(2,0,0,2,4,6)"><rect x="2" y="3" width="4" height="5" fill="rgb(101,119,255)"/><path d="M1 1L2 2Z" fill="rgb(101,119,254)"/><path d="M2 2L3 3Z" fill="rgb(101,119,253)"/></g></svg>"#;
        let output = convert_svg(source, false).expect("svg");

        assert!(output.contains(
            "Path d:\"M2 3H6V8H2Z\" fill:\"secondary\" transform:\"matrix(2 0 0 2 4 6)\""
        ));
        assert_eq!(output.matches("fill:\"secondary\"").count(), 2);
        assert_eq!(output.matches("fill:\"accent\"").count(), 1);
    }

    #[test]
    fn rejects_svg_without_portable_paths() {
        assert!(
            convert_svg(
                r#"<svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="5"/></svg>"#,
                false
            )
            .is_err()
        );
    }

    #[test]
    fn preserves_original_hex_colors_and_builds_preview_data() {
        let source = r##"<svg viewBox="0 0 20 10"><path d="M0 0H10V10Z" fill="#000000"/><path d="M10 0H20V10Z" fill="rgb(107,198,112)"/></svg>"##;
        let output = convert_svg(source, true).expect("source");
        let data = convert_svg_data(source).expect("data");

        assert!(output.contains("fill:\"#000000\""));
        assert!(output.contains("fill:\"#6bc670\""));
        let data = serde_json::from_str::<Value>(data.as_str().expect("json")).expect("record");
        assert_eq!(data["viewBox"], "0 0 20 10");
        assert_eq!(data["paths"][0]["color"], "#000000");
        assert_eq!(data["paths"][1]["color"], "#6bc670");
    }

    #[test]
    fn preserves_inherited_evenodd_fill_rule_for_source_and_preview_data() {
        let source = r##"<svg viewBox="0 0 24 24" style="fill-rule:evenodd;clip-rule:evenodd"><g transform="matrix(1,0,0,1,2,3)"><path d="M0 0H20V20H0ZM4 4H16V16H4Z" fill="#6bc66e"/></g><g fill-rule="nonzero"><path d="M1 1H2V2Z" fill="#1f3a60"/></g></svg>"##;
        let output = convert_svg(source, true).expect("source");
        let data = convert_svg_data(source).expect("data");

        assert!(
            output.contains(
                "fill:\"#6bc66e\" fillRule:\"evenodd\" transform:\"matrix(1 0 0 1 2 3)\""
            )
        );
        assert!(!output.contains("fill:\"#1f3a60\" fillRule:"));
        let data = serde_json::from_str::<Value>(data.as_str().expect("json")).expect("record");
        assert_eq!(data["paths"][0]["evenOdd"], true);
        assert!(data["paths"][1].get("evenOdd").is_none());
    }
}
