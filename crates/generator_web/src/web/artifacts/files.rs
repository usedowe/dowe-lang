pub fn web_artifacts(
    web: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
) -> Vec<WebArtifact> {
    web_artifacts_for_target(web, font_config, design_config, Path::new(""), "web")
}

pub fn web_artifacts_for_target(
    web: &WebOutput,
    font_config: &FontConfig,
    design_config: &DesignConfig,
    prefix: &Path,
    target: &'static str,
) -> Vec<WebArtifact> {
    let mut artifacts = Vec::new();
    let design_css = design_css_for_web(web, font_config, design_config);
    let design_file_name = design_css_file_name(&design_css);

    for chunk in &web.chunks {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.relative_path),
            content: chunk.content.clone(),
            kind: WebArtifactKind::Chunk,
            target,
        });
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.css_relative_path),
            content: chunk.css_content.clone(),
            kind: WebArtifactKind::Css,
            target,
        });
    }

    for chunk in &web.translation_chunks {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.relative_path),
            content: chunk.content.clone(),
            kind: WebArtifactKind::Chunk,
            target,
        });
    }

    for chunk in web.runtime_chunks() {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.relative_path),
            content: chunk.content,
            kind: WebArtifactKind::Chunk,
            target,
        });
    }

    for chunk in design_css_chunks_for_web(web) {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, &chunk.relative_path),
            content: chunk.content,
            kind: WebArtifactKind::Css,
            target,
        });
    }

    artifacts.push(WebArtifact {
        relative_path: prefixed_path(prefix, &Path::new("web").join(&design_file_name)),
        content: design_css,
        kind: WebArtifactKind::Css,
        target,
    });

    artifacts.push(WebArtifact {
        relative_path: prefixed_path(
            prefix,
            &Path::new("web").join(prepared_router_file_name(web)),
        ),
        content: web.router_js.clone(),
        kind: WebArtifactKind::Chunk,
        target,
    });

    artifacts.push(WebArtifact {
        relative_path: prefixed_path(prefix, Path::new("web/manifest.json")),
        content: manifest(web),
        kind: WebArtifactKind::Manifest,
        target,
    });

    if let Some(page) = web.pages.first() {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(prefix, Path::new("web/index.html")),
            content: static_html_document(&page.html_document, ""),
            kind: WebArtifactKind::Html,
            target,
        });
    }

    for page in &web.pages {
        artifacts.push(WebArtifact {
            relative_path: prefixed_path(
                prefix,
                Path::new(&format!("web/pages/{}.html", page_file_name(page))),
            ),
            content: static_html_document(&page.html_document, "../"),
            kind: WebArtifactKind::Html,
            target,
        });
    }

    artifacts
}

fn prefixed_path(prefix: &Path, path: &Path) -> PathBuf {
    if prefix.as_os_str().is_empty() {
        path.to_path_buf()
    } else {
        prefix.join(path)
    }
}

fn static_html_document(document: &str, asset_prefix: &str) -> String {
    let document = document
        .replace(
            r#"href="/design.css""#,
            &format!(r#"href="{asset_prefix}design.css""#),
        )
        .replace(
            r#"href="/design-"#,
            &format!(r#"href="{asset_prefix}design-"#),
        )
        .replace(
            r#"href="/chunks/"#,
            &format!(r#"href="{asset_prefix}chunks/"#),
        )
        .replace(
            r#"src="/router-"#,
            &format!(r#"src="{asset_prefix}router-"#),
        )
        .replace(
            r#"src="/router.js""#,
            &format!(r#"src="{asset_prefix}router.js""#),
        )
        .replace(
            r#"src="/chunks/"#,
            &format!(r#"src="{asset_prefix}chunks/"#),
        );
    rewrite_static_route_hrefs(&document, asset_prefix)
}

fn rewrite_static_route_hrefs(document: &str, asset_prefix: &str) -> String {
    let mut output = String::new();
    let mut rest = document;

    while let Some(index) = rest.find(r#" href="/"#) {
        output.push_str(&rest[..index]);
        let value_start = index + r#" href=""#.len();
        let Some(value_end_offset) = rest[value_start..].find('"') else {
            output.push_str(&rest[index..]);
            return output;
        };
        let value_end = value_start + value_end_offset;
        let href = &rest[value_start..value_end];
        output.push_str(r#" href=""#);
        output.push_str(&static_route_href(href, asset_prefix));
        output.push('"');
        rest = &rest[value_end + 1..];
    }

    output.push_str(rest);
    output
}

fn static_route_href(href: &str, asset_prefix: &str) -> String {
    let (path, fragment) = href.split_once('#').unwrap_or((href, ""));
    if let Some(asset) = path.strip_prefix("/assets/") {
        return format!("{asset_prefix}assets/{asset}");
    }
    if let Some(icon) = path.strip_prefix("/icons/") {
        return format!("{asset_prefix}icons/{icon}");
    }
    let file = if path == "/" {
        if asset_prefix.is_empty() {
            "index.html".to_string()
        } else {
            "../index.html".to_string()
        }
    } else {
        let name = format!("{}.html", path.trim_matches('/').replace('/', "-"));
        if asset_prefix.is_empty() {
            format!("pages/{name}")
        } else {
            name
        }
    };

    if fragment.is_empty() {
        file
    } else {
        format!("{file}#{fragment}")
    }
}

