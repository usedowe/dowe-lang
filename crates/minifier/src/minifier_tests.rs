#[cfg(test)]
mod tests {
    use super::{minify_css, minify_js};

    #[test]
    fn minifies_generated_javascript() {
        let input = r#"
            export const value = "Layout";
            export function render(children = "") {
                return "<div>" + children + "</div>";
            }
        "#;

        assert_eq!(
            minify_js(input),
            r#"export const value="Layout";export function render(children=""){return"<div>"+children+"</div>";}"#
        );
    }

    #[test]
    fn removes_javascript_comments() {
        let input = r#"
            const value = "http://localhost";
            const next = 1;
        "#;

        assert_eq!(
            minify_js(input),
            r#"const value="http://localhost";const next=1;"#
        );
    }

    #[test]
    fn preserves_escaped_slashes_in_regular_expressions() {
        let input = r#"
            const path = value.replace(/^web\//, "");
            const next = 1;
        "#;

        assert_eq!(
            minify_js(input),
            r#"const path=value.replace(/^web\//,"");const next=1;"#
        );
    }

    #[test]
    fn preserves_spaces_between_ambiguous_operators() {
        assert_eq!(minify_js("const x = a + +b;"), "const x=a+ +b;");
        assert_eq!(minify_js("const x = a - -b;"), "const x=a- -b;");
    }

    #[test]
    fn preserves_division_before_a_regular_expression() {
        assert_eq!(
            minify_js("const ok = value / /a/.test(text);"),
            "const ok=value/ /a/.test(text);"
        );
    }

    #[test]
    fn preserves_restricted_line_breaks() {
        assert_eq!(minify_js("return\n{ ok: true };"), "return\n{ok:true};");
        assert_eq!(minify_js("value\n++next;"), "value\n++next;");
    }

    #[test]
    fn minifies_template_interpolations_without_changing_literal_text() {
        assert_eq!(
            minify_js("const value = `Hello  ${ name || \"Dowe\" }`;"),
            "const value=`Hello  ${name||\"Dowe\"}`;"
        );
    }

    #[test]
    fn minifies_css() {
        let input = r#"
            .box {
                color: red;
                padding: 8px;
            }
        "#;

        assert_eq!(minify_css(input), ".box{color:red;padding:8px;}");
    }

    #[test]
    fn preserves_css_math_operator_whitespace() {
        let input = r#"
            .box {
                width: calc(100% - 2rem);
                height: min(50vh + 1rem, 40rem);
            }
        "#;

        assert_eq!(
            minify_css(input),
            ".box{width:calc(100% - 2rem);height:min(50vh + 1rem,40rem);}",
        );
    }

    #[test]
    fn preserves_css_strings_urls_and_custom_property_values() {
        let input = r#"
            :root {
                --dowe-font-stack: "Dowe Sans", system-ui, sans-serif;
                --dowe-offset: calc(100% - 2rem);
            }
            .icon {
                content: "a /* literal */ b";
                background: url("data:image/svg+xml,%3Csvg viewBox='0 0 2 2'%3E%3C/svg%3E");
            }
        "#;

        assert_eq!(
            minify_css(input),
            ":root{--dowe-font-stack:\"Dowe Sans\",system-ui,sans-serif;--dowe-offset:calc(100% - 2rem);}.icon{content:\"a /* literal */ b\";background:url(\"data:image/svg+xml,%3Csvg viewBox='0 0 2 2'%3E%3C/svg%3E\");}",
        );
    }

    #[test]
    fn removes_css_comments_without_joining_identifiers() {
        assert_eq!(
            minify_css(".box { font: 600 /* generated */ 1rem sans-serif; }"),
            ".box{font:600 1rem sans-serif;}",
        );
    }
}

