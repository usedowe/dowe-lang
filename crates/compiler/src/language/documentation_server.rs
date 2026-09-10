pub(super) fn server_names() -> impl Iterator<Item = &'static str> {
    SERVER_DOCUMENTATION.iter().map(|entry| entry.name)
}

pub(super) fn server_props(name: &str) -> Vec<&'static str> {
    SERVER_DOCUMENTATION
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| {
            entry
                .signature
                .split_whitespace()
                .filter_map(|token| {
                    let token = token.trim_matches(['[', ']']);
                    token.split_once(':').map(|(name, _)| name)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn component_documentation(name: &str) -> Option<String> {
    if name == "validate" {
        return Some(
            "## `validate`\n\nDeclares one ordered client-side validation rule inside `Input`, `Date`, `Pin`, `Phone`, `Select` or `Checkbox`, or validates a Signal globally inside a view function with `validate signalName`. The first failing rule supplies the visible error after the control is touched.\n\n**Accepted props**\n\n- `rule`: quoted rule identifier\n- `message`: quoted non-empty error message"
                .to_string(),
        );
    }
    BuiltinComponent::from_name(name)?;
    let props = props_for_component(name);
    let mut output = format!(
        "## `{name}`\n\n{}\n\n**Accepted props**\n",
        component_description(name)
    );
    for prop in props {
        output.push_str(&format!("\n- `{prop}`: {}", prop_type(name, prop)));
    }
    let children = component_children(name);
    if !children.is_empty() {
        output.push_str("\n\n**Accepted children**\n");
        for (child, description) in children {
            output.push_str(&format!("\n- `{child}` {description}"));
        }
    }
    Some(output)
}

pub(super) fn component_prop_documentation(component: &str, prop: &str) -> Option<String> {
    let props = props_for_component(component);
    (!props.is_empty() && props.contains(&prop)).then(|| {
        let value_type = prop_type(component, prop);
        let description = if component == "Icon" && prop == "name" {
            "Selects a member of the shared Solar, country-flag, SVG Spinner, or SVG Logos catalog. A bare path must resolve to a string; Signal changes update the icon and invalid runtime values use the validated initial icon."
        } else if (component == "Image" || component == "Iframe") && prop == "src" {
            "Accepts a quoted packaged asset, HTTPS URL, or internal route, or a bare path resolving to a string constant, Signal, or each-item value. The compiler validates the path and string type before lowering it for every target."
        } else {
            prop_description(component, prop)
        };
        format!(
            "### `{component}.{prop}`\n\n**Type:** `{value_type}`\n\n{}",
            description
        )
    })
}

pub(super) fn theme_documentation(owner: &str, token: &str, root_theme: bool) -> Option<String> {
    match (owner, token, root_theme) {
        ("theme", "theme", true) => Some(
            "## `theme`\n\n```dowe\ntheme\n  fonts default:\"inter\" install:[\"inter\"]\n  design defaultTheme:\"light\"\n```\n\nDeclares the canonical project theme configuration in `theme.dowe`.\n\n**Accepted children**\n\n- `app` for visible application metadata\n- `fonts` for the default and installed font families\n- `design` for the default theme, component defaults, and named themes"
                .to_string(),
        ),
        ("theme", "theme", false) => Some(
            "## `theme`\n\n```dowe\ntheme name:\"brand\" extends:\"light\"\n  colors:\n    primary color:\"#2563eb\" text:\"#ffffff\" title:\"#fffffe\"\n```\n\nDeclares a named color theme inside `design`. Each grouped semantic family declares `color`, `text`, and `title`: the base value, ordinary content and controls, and titles and semantic headers respectively. A named theme may inherit from another named or built-in theme and override any role within a family. Component defaults belong in the component entries under `design`; explicit usage props take precedence over those defaults.\n\n**Accepted props**\n\n- `name`: stable lowercase theme name\n- `extends`: optional theme name to inherit\n\n**Accepted children**\n\n- `colors` with grouped semantic color families"
                .to_string(),
        ),
        (owner, token, _)
            if !matches!(owner, "design" | "fonts")
                && ColorFamily::from_theme_name(owner).is_some()
                && owner == token =>
        {
            Some(format!(
                "## `{owner}` color family\n\nDeclares one grouped semantic color family. Its `color`, `text`, and `title` props are normalized into the shared target-neutral color tokens. In an inherited theme, any omitted role comes from the parent theme."
            ))
        }
        (owner, "color", _) if ColorFamily::from_theme_name(owner).is_some() => Some(
            "### `color`\n\nBase semantic color used as the filled surface or family accent."
                .to_string(),
        ),
        (owner, "text", _) if ColorFamily::from_theme_name(owner).is_some() => Some(
            "### `text`\n\nSemantic color for ordinary content, control labels, and authored `Text` inside the family surface."
                .to_string(),
        ),
        (owner, "title", _) if ColorFamily::from_theme_name(owner).is_some() => Some(
            "### `title`\n\nSemantic color for authored `Title` and integrated semantic headers inside the family surface."
                .to_string(),
        ),
        ("design", "design", _) => Some(
            "## `design`\n\n```dowe\ndesign defaultTheme:\"light\"\n  Button variant:\"outlined\"\n  Input variant:\"outlined\" scheme:\"primary\"\n  Text font:\"manrope\"\n  Title font:\"syne\"\n  theme name:\"light\"\n```\n\nConfigures the default named color theme and static visual defaults that Dowe injects into the shared view model. The precedence is explicit usage prop, then the matching `design` entry, then the built-in component default. Built-in defaults intentionally add no border or shadow unless the project configures those props. The normalized defaults are shared by web, desktop, Android, and iOS output.\n\n**Accepted props**\n\n- `defaultTheme`: declared theme name used initially\n\n**Accepted children**\n\n- `Button`, `IconButton`, `Card`, `Drawer`, `Toast`, `Section`, `Accordion`, `Tree`, `Checkbox`, `Input`, `Date`, `Password`, `Select`, `Pin`, `AppBar`, `Footer`, `Modal`, `Dropdown`, `Tooltip`, `Tabs`\n- `Chip`, `SideNav`, `Sidebar`, `NavMenu`, `Avatar`, and `Ui` for existing shared visual defaults\n- `Text` for the default text font\n- `Title` for the default title font\n- `theme` for named color tokens"
                .to_string(),
        ),
        ("Tabs", "Tabs", _) => Some(
            "## `Tabs` theme defaults\n\n```dowe\nTabs variant:\"pills\" scheme:\"primary\"\n```\n\nDeclares optional static defaults for `Tabs` inside `design`. Its `variant` accepts `solid`, `outlined`, `line`, `ghost`, or `pills`; the built-in default is `pills` with the `primary` scheme. An explicit prop on a component usage always wins; omitted props retain Dowe's built-in component defaults."
                .to_string(),
        ),
        (component @ ("Card" | "Button" | "IconButton" | "Drawer" | "Toast" | "Section" | "Accordion" | "Checkbox" | "Input" | "Date" | "Password" | "Select" | "Pin" | "AppBar" | "Footer" | "Modal" | "Dropdown" | "Tooltip" | "Chip" | "Avatar" | "Ui"), token, _)
            if component == token => Some(
            format!(
                "## `{component}` theme defaults\n\n```dowe\n{component} variant:\"outline\" scheme:\"primary\" radius:\"xs\" shadow:\"xs\"\n```\n\nDeclares optional static defaults for `{component}` inside `design`. An explicit prop on a component usage always wins; omitted props retain Dowe's built-in component defaults.\n\n**Accepted props**\n\n- `variant`: `solid`, `outline`, `outlined`, `line`, or `ghost`\n- `scheme`: semantic color family\n- `radius` or `rounded`: `xs`, `sm`, `md`, `lg`, `xl`, or `full`\n- `shadow`: `xs`, `sm`, `md`, `lg`, or `xl`\n- `shadowColor`: semantic color family\n- `border`: integer from `1` to `4`\n- `borderColor`: semantic color family\n- `size`: `xs`, `sm`, `md`, `lg`, or `xl`"
            ),
        ),
        (component @ ("Text" | "Title"), token, _) if component == token => Some(format!(
            "## `{component}` theme defaults\n\n```dowe\n{component} font:\"manrope\"\n```\n\nDeclares the project-wide default font for `{component}` inside `design`. A `font` prop on one component instance always wins. The configured family is included in generated font assets.\n\n**Accepted props**\n\n- `font`: one quoted Dowe font token"
        )),
        ("fonts", "fonts", _) => Some(
            "## `fonts`\n\n```dowe\nfonts default:\"inter\" install:[\"inter\"]\n```\n\nConfigures project font families from Dowe's built-in catalog. The default family is included in generated targets even when it is absent from `install`.\n\n**Accepted props**\n\n- `default`: one quoted font token; defaults to `\"inter\"`\n- `install`: ordered array of additional quoted font tokens\n\n**Font tokens**\n\n`\"system\"`, `\"inter\"`, `\"roboto\"`, `\"montserrat\"`, `\"lato\"`, `\"poppins\"`, `\"manrope\"`, `\"quicksand\"`, `\"lora\"`, `\"syne\"`, `\"jost\"`, `\"puritan\"`"
                .to_string(),
        ),
        ("fonts", "default", _) => Some(
            "### `fonts.default`\n\n**Type:** quoted font token\n\nSelects the project-wide default font. Dowe uses `\"inter\"` when this prop is omitted."
                .to_string(),
        ),
        ("fonts", "install", _) => Some(
            "### `fonts.install`\n\n**Type:** array of quoted font tokens\n\nAdds font families to the effective generated font set even when no View uses them directly. Values must be unique."
                .to_string(),
        ),
        _ => None,
    }
}

pub(super) fn server_documentation(name: &str) -> Option<String> {
    let entry = SERVER_DOCUMENTATION
        .iter()
        .find(|entry| entry.name == name)?;
    let mut output = format!(
        "## `{}`\n\n```dowe\n{}\n```\n\n{}",
        entry.name, entry.signature, entry.description
    );
    if entry.name == "main" {
        output.push_str(
            "\n\n**Accepted props**\n\n- None\n\n**Accepted children**\n\n- `app` (`name` and `bundle` metadata)\n- `views:<symbol|array>` (one or more imported view route graphs)\n- `server` (optional server target)\n- `desktop` (optional desktop server container)",
        );
    }
    Some(output)
}

pub(super) fn server_prop_documentation(line: &str, prop: &str) -> Option<String> {
    let marker = format!("{prop}:");
    let entry = SERVER_DOCUMENTATION
        .iter()
        .filter(|entry| line.split_whitespace().any(|token| token == entry.name))
        .find(|entry| entry.signature.contains(&marker))?;
    Some(format!(
        "### `{}.{prop}`\n\n```dowe\n{}\n```\n\nAccepted and validated by the shared Dowe server compiler.",
        entry.name, entry.signature
    ))
}

pub(super) fn server_owner_prop_documentation(owner: &str, prop: &str) -> Option<String> {
    let entry = SERVER_DOCUMENTATION
        .iter()
        .find(|entry| entry.name == owner && server_props(owner).contains(&prop))?;
    Some(format!(
        "### `{owner}.{prop}`\n\n```dowe\n{}\n```\n\nAccepted and validated by the shared Dowe server compiler.",
        entry.signature
    ))
}

pub(super) fn stdlib_documentation(name: &str) -> Option<String> {
    let (namespace, function) = name.split_once('.')?;
    let signature = dowe_stdlib::signature(namespace, function)?;
    Some(format_stdlib(&signature))
}

fn format_stdlib(signature: &StdlibSignature) -> String {
    let mut args = signature
        .required
        .iter()
        .map(|name| format!("{name}:<value>"))
        .collect::<Vec<_>>();
    args.extend(
        signature
            .optional
            .iter()
            .map(|name| format!("[{name}:<value>]")),
    );
    let name = format!("{}.{}", signature.namespace, signature.function);
    format!(
        "## `{name}`\n\n```dowe\n{name} {}\n```\n\n**Returns:** `{}`\n\n{}",
        args.join(" "),
        return_kind(signature.return_kind),
        signature.description
    )
}

