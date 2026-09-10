fn customize_main_file(file: TemplateFile, options: &InitProjectOptions) -> TemplateFile {
    let (Some(app_name), Some(bundle)) = (options.app_name(), options.bundle()) else {
        return file;
    };
    if file.path() != "main.dowe" {
        return file;
    }
    let identity = format!(
        "app name:\"{}\" bundle:\"{}\"",
        escape_string(app_name),
        escape_string(bundle)
    );
    let content = file
        .content()
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("app name:") {
                let indentation = &line[..line.len() - line.trim_start().len()];
                format!("{indentation}{identity}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let content = if file.content().ends_with('\n') {
        format!("{content}\n")
    } else {
        content
    };
    TemplateFile::owned(file.path(), content)
}

fn escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn files_with_translations(
    mut files: Vec<TemplateFile>,
    options: InitProjectOptions,
    translations: &[InitTranslation],
) -> Vec<TemplateFile> {
    if options.i18n_enabled() {
        files = files
            .into_iter()
            .map(|file| localize_template_file(file, translations))
            .collect();
        files.push(TemplateFile::owned(
            "i18n/en.dowe",
            render_translation_catalog(translations, true, |entry| entry.en),
        ));
        files.push(TemplateFile::owned(
            "i18n/es.dowe",
            render_translation_catalog(translations, false, |entry| entry.es),
        ));
    }
    files
}

fn localize_template_file(file: TemplateFile, translations: &[InitTranslation]) -> TemplateFile {
    if !file.path().starts_with("views/") || !file.path().ends_with(".dowe") {
        return file;
    }
    TemplateFile::owned(
        file.path(),
        localize_view_source(file.content(), translations),
    )
}

fn localize_view_source(source: &str, translations: &[InitTranslation]) -> String {
    let mut lines = source.lines().map(str::to_owned).collect::<Vec<_>>();
    for index in 0..lines.len().saturating_sub(1) {
        let component = lines[index]
            .trim_start()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !matches!(component, "Text" | "Title" | "Button") {
            continue;
        }
        let fallback = lines[index + 1].trim();
        let Some(fallback) = fallback
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
        else {
            continue;
        };
        let Some(translation) = translations.iter().find(|entry| entry.en == fallback) else {
            continue;
        };
        lines[index].push_str(&format!(" i18n:\"{}\"", translation.key));
    }
    let mut localized = lines.join("\n");
    if source.ends_with('\n') {
        localized.push('\n');
    }
    localized
}

fn render_translation_catalog(
    translations: &[InitTranslation],
    default: bool,
    value: impl Fn(&InitTranslation) -> &'static str,
) -> String {
    let mut catalog = if default {
        "translations default:true\n".to_string()
    } else {
        "translations\n".to_string()
    };
    let mut current_group = None;
    for translation in translations {
        let (group, leaf) = translation
            .key
            .split_once('.')
            .expect("init translation key group");
        assert!(!leaf.contains('.'), "init translation key depth");
        if current_group != Some(group) {
            catalog.push_str(&format!("  {group}\n"));
            current_group = Some(group);
        }
        catalog.push_str(&format!(
            "    {leaf} \"{}\"\n",
            escape_translation_value(value(translation))
        ));
    }
    catalog
}

fn escape_translation_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

