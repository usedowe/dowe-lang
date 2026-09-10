#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_translation_catalog() {
        let (_, is_default, values) = parse_catalog(
            r#"translations default:true
  common
    welcome "Welcome, {{name}}!"
  messages
    items
      singular "You have {{count}} item"
      plural "You have {{count}} items"
"#,
        );

        assert!(is_default);
        assert_eq!(value(&values, "common.welcome"), Some("Welcome, {{name}}!"));
        assert_eq!(
            value(&values, "messages.items.singular"),
            Some("You have {{count}} item")
        );
        assert_eq!(
            value(&values, "messages.items.plural"),
            Some("You have {{count}} items")
        );
    }

    #[test]
    fn keeps_explicit_translation_entries_compatible() {
        let (_, _, values) = parse_catalog(
            r#"translations default:true
  translation key:"home.hero.title" value:"Dowe builds systems."
"#,
        );

        assert_eq!(
            value(&values, "home.hero.title"),
            Some("Dowe builds systems.")
        );
    }

    #[test]
    fn rejects_duplicate_nested_and_explicit_keys() {
        let root = Path::new("/project");
        let path = root.join("i18n/en.dowe");
        let file = parse_source_file(
            root,
            &path,
            r#"translations default:true
  home
    hero
      title "Dowe builds systems."
  translation key:"home.hero.title" value:"Duplicate"
"#
            .to_string(),
        )
        .expect("source");

        let error = parse_translation_file(&file).expect_err("duplicate");

        assert!(error.to_string().contains("duplicate translation key"));
    }

    fn parse_catalog(source: &str) -> (String, bool, Vec<TranslationValue>) {
        let root = Path::new("/project");
        let path = root.join("i18n/en.dowe");
        let file = parse_source_file(root, &path, source.to_string()).expect("translation source");
        parse_translation_file(&file).expect("translation catalog")
    }

    fn value<'a>(values: &'a [TranslationValue], key: &str) -> Option<&'a str> {
        values
            .iter()
            .find(|value| value.key == key)
            .map(|value| value.value.as_str())
    }
}
