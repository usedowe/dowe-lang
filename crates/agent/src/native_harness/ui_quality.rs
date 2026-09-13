use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct UiFinding {
    pub path: PathBuf,
    pub line: usize,
    pub component: String,
    pub prop: String,
    pub value: String,
    pub message: &'static str,
}

#[derive(Default)]
pub(crate) struct UiQualityAudit {
    findings: Vec<UiFinding>,
    horizontal_nav: Option<(PathBuf, usize)>,
    h1_titles: Vec<(PathBuf, usize)>,
    has_app_bar: bool,
    has_drawer: bool,
    has_side_nav: bool,
    has_mobile_trigger: bool,
}

impl UiQualityAudit {
    pub(crate) fn inspect(&mut self, path: &Path, content: &str) {
        for (line_index, line) in content.lines().enumerate() {
            let Some(component) = component_name(line) else {
                continue;
            };
            let line_number = line_index + 1;
            match component {
                "AppBar" => self.has_app_bar = true,
                "Drawer" => self.has_drawer = true,
                "SideNav" => self.has_side_nav = true,
                "IconButton" => {
                    if mobile_only_visibility(line) {
                        self.has_mobile_trigger = true;
                    }
                }
                "NavMenu" => {
                    if self.horizontal_nav.is_none() {
                        self.horizontal_nav = Some((path.to_path_buf(), line_number));
                    }
                    if property_value(line, "variant").as_deref() == Some("solid") {
                        self.push(path, line_number, component, "variant", "solid", "NavMenu is horizontal navigation; use its ghost default instead of a solid control surface");
                    }
                }
                "Text" => {
                    self.inspect_text(path, line_number, line);
                }
                "Title" => {
                    self.inspect_title(path, line_number, line);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn finish(mut self, max_findings: usize) -> Vec<UiFinding> {
        if let Some((path, line)) = self.horizontal_nav.clone()
            && (!self.has_drawer || !self.has_side_nav || !self.has_mobile_trigger)
        {
            self.push(
                &path,
                line,
                "NavMenu",
                "mobileNavigation",
                "missing",
                "reference UI NavMenu requires a mobile IconButton, shell Drawer, and vertical SideNav",
            );
        }
        if self.horizontal_nav.is_some() && !self.has_app_bar {
            let (path, line) = self
                .horizontal_nav
                .clone()
                .expect("desktop navigation reference");
            self.push(
                &path,
                line,
                "NavMenu",
                "shell",
                "missing AppBar",
                "place horizontal NavMenu directly in an AppBar region owned by a Scaffold layout",
            );
        }
        let extra_h1_titles = self.h1_titles.iter().skip(1).cloned().collect::<Vec<_>>();
        for (path, line) in extra_h1_titles {
            self.push(
                &path,
                line,
                "Title",
                "as",
                "h1",
                "reference UI pages may use Title as h1 only once; keep other headings at the default h2 semantic",
            );
        }
        self.findings.truncate(max_findings);
        self.findings
    }

    fn inspect_text(&mut self, path: &Path, line: usize, source: &str) {
        if let Some(value) = property_value(source, "color") {
            self.push(
                path,
                line,
                "Text",
                "color",
                &value,
                "Text color is inherited from the parent scheme; remove the local color prop",
            );
        }
        if let Some(value) = property_value(source, "size")
            && value == "xs"
        {
            self.push(
                path,
                line,
                "Text",
                "size",
                &value,
                "reference UI text must use the theme/default scale; remove Text size xs",
            );
        }
        if let Some(value) = property_value(source, "weight") {
            self.push(path, line, "Text", "weight", &value, "reference UI text weight belongs to the theme and composition; remove the local weight prop");
        }
    }

    fn inspect_title(&mut self, path: &Path, line: usize, source: &str) {
        if let Some(value) = property_value(source, "color") {
            self.push(
                path,
                line,
                "Title",
                "color",
                &value,
                "Title color is inherited from the parent scheme; remove the local color prop",
            );
        }
        if let Some(value) = property_value(source, "weight") {
            self.push(
                path,
                line,
                "Title",
                "weight",
                &value,
                "Title already owns its heading weight; remove the local weight prop",
            );
        }
        if responsive_size(source) {
            self.push(
                path,
                line,
                "Title",
                "size",
                "responsive",
                "Title size is fluid from a scalar token; remove the responsive size object",
            );
        }
        if property_value(source, "as").as_deref() == Some("h1") {
            self.h1_titles.push((path.to_path_buf(), line));
        }
    }

    fn push(
        &mut self,
        path: &Path,
        line: usize,
        component: &str,
        prop: &str,
        value: &str,
        message: &'static str,
    ) {
        self.findings.push(UiFinding {
            path: path.to_path_buf(),
            line,
            component: component.to_string(),
            prop: prop.to_string(),
            value: value.to_string(),
            message,
        });
    }
}

fn component_name(line: &str) -> Option<&str> {
    let name = line.split_whitespace().next()?;
    if name.is_empty()
        || !name.chars().next()?.is_ascii_uppercase()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }
    Some(name)
}

fn property_value(line: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}:");
    line.split_whitespace()
        .skip(1)
        .find_map(|token| token.strip_prefix(&prefix))
        .map(|value| value.trim_matches(['"', '\'', ',']).to_string())
}

fn responsive_size(line: &str) -> bool {
    property_value(line, "size").is_some_and(|value| value == "{")
        || line.contains("size:{")
        || line.contains("size: {")
}

fn mobile_only_visibility(line: &str) -> bool {
    property_value(line, "show").is_some_and(|value| value == "{")
        && line.contains("xs:true")
        && line.contains("md:false")
}
