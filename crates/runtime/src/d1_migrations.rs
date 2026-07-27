use crate::{RuntimeError, RuntimeResult};
use dowe_components::{RuntimeSvgCatalogEntry, solar_runtime_svg_catalog};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1IconCatalogMigrationReport {
    pub migrations: usize,
    pub icon_variants: usize,
    pub output: PathBuf,
}

pub fn generate_solar_icon_catalog_d1_migrations(
    project_root: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> RuntimeResult<D1IconCatalogMigrationReport> {
    let output = project_relative_output(project_root.as_ref(), output.as_ref())?;
    fs::create_dir_all(&output)?;
    let catalog =
        solar_runtime_svg_catalog().map_err(|error| RuntimeError::new(error.to_string()))?;
    let migrations = catalog_d1_migrations(&catalog);
    for (name, sql) in &migrations {
        fs::write(output.join(name), sql)?;
    }
    Ok(D1IconCatalogMigrationReport {
        migrations: migrations.len(),
        icon_variants: catalog.len(),
        output,
    })
}

fn project_relative_output(project_root: &Path, output: &Path) -> RuntimeResult<PathBuf> {
    if output.as_os_str().is_empty()
        || output.is_absolute()
        || output
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(RuntimeError::new(
            "--output must stay inside the current project",
        ));
    }
    Ok(project_root.join(output))
}

fn catalog_d1_migrations(catalog: &[RuntimeSvgCatalogEntry]) -> Vec<(String, String)> {
    let mut entries = catalog.to_vec();
    entries.sort_by(|left, right| {
        left.category
            .cmp(right.category)
            .then(left.name.cmp(right.name))
            .then(left.style.cmp(right.style))
    });
    let mut categories = BTreeMap::<&str, BTreeSet<&str>>::new();
    for entry in &entries {
        categories
            .entry(entry.category)
            .or_default()
            .insert(entry.name);
    }
    let mut schema = String::from(
        "CREATE TABLE IF NOT EXISTS icon_categories (\n  slug TEXT PRIMARY KEY,\n  label TEXT NOT NULL,\n  position INTEGER NOT NULL,\n  count INTEGER NOT NULL\n);\n\nCREATE TABLE IF NOT EXISTS icons (\n  id TEXT PRIMARY KEY,\n  name TEXT NOT NULL,\n  category TEXT NOT NULL,\n  style TEXT NOT NULL,\n  svg TEXT NOT NULL,\n  UNIQUE(name, style),\n  FOREIGN KEY(category) REFERENCES icon_categories(slug)\n);\n\nCREATE INDEX IF NOT EXISTS icons_category_style_name ON icons(category, style, name);\nCREATE INDEX IF NOT EXISTS icons_name ON icons(name);\n\n",
    );
    for (position, (slug, names)) in categories.iter().enumerate() {
        schema.push_str(&format!(
            "INSERT OR REPLACE INTO icon_categories (slug, label, position, count) VALUES ('{}', '{}', {}, {});\n",
            sql_escape(slug),
            sql_escape(&category_label(slug)),
            position,
            names.len()
        ));
    }
    let mut migrations = vec![("00001_icon_catalog.sql".to_string(), schema)];
    for (index, chunk) in entries.chunks(200).enumerate() {
        let mut sql = String::new();
        for entry in chunk {
            sql.push_str(&format!(
                "INSERT OR REPLACE INTO icons (id, name, category, style, svg) VALUES ('{}', '{}', '{}', '{}', '{}');\n",
                sql_escape(&format!("solar:{}:{}", entry.style, entry.name)),
                sql_escape(entry.name),
                sql_escape(entry.category),
                sql_escape(entry.style),
                sql_escape(&entry.svg)
            ));
        }
        migrations.push((format!("{:05}_solar_icons.sql", index + 2), sql));
    }
    migrations
}

fn sql_escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn category_label(value: &str) -> String {
    value
        .split('-')
        .map(|part| match part {
            "it" => "IT".to_string(),
            "ui" => "UI".to_string(),
            part => {
                let mut characters = part.chars();
                characters
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
                    .unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{catalog_d1_migrations, project_relative_output};
    use dowe_components::solar_runtime_svg_catalog;
    use std::path::Path;

    #[test]
    fn generates_complete_chunked_d1_catalog() {
        let catalog = solar_runtime_svg_catalog().expect("catalog");
        let migrations = catalog_d1_migrations(&catalog);
        assert_eq!(catalog.len(), 7476);
        assert_eq!(migrations.len(), 39);
        assert!(migrations[0].1.contains("CREATE TABLE IF NOT EXISTS icons"));
        assert_eq!(
            migrations
                .iter()
                .skip(1)
                .map(|(_, sql)| sql.matches("INSERT OR REPLACE INTO icons").count())
                .sum::<usize>(),
            7476
        );
    }

    #[test]
    fn rejects_output_outside_the_project() {
        let error = project_relative_output(Path::new("/project"), Path::new("../migrations"))
            .expect_err("parent path error");
        assert!(error.to_string().contains("inside the current project"));
    }
}
