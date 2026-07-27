fn write_spec_fixture(root: &Path, with_acceptance: bool) {
    let spec_root = root.join("specs/features/00001-example-feature");
    fs::create_dir_all(&spec_root).expect("spec root");
    fs::write(
        spec_root.join("spec.md"),
        "# Example Feature\n\nStatus: Draft\n\n## Resumen\n\nImplement behavior.\n",
    )
    .expect("spec");
    fs::write(
        spec_root.join("compiler.md"),
        "# Compiler Contract\n\n## Modelo\n\nShared behavior.\n",
    )
    .expect("compiler");
    if with_acceptance {
        fs::write(
            spec_root.join("acceptance.md"),
            "# Criterios de aceptación\n\n- Creates the artifact.\n- Rejects invalid paths.\n",
        )
        .expect("acceptance");
    }
}

fn write_complex_spec_fixture(root: &Path) {
    let spec_root = root.join("specs/features/00002-complex-feature");
    fs::create_dir_all(&spec_root).expect("spec root");
    fs::write(
        spec_root.join("spec.md"),
        "# Complex Feature\n\nStatus: Draft\n\n## Resumen\n\nAdd a new crate and pipeline behavior.\n\n## Plan Modular\n\nOwner and files are declared here.\n",
    )
    .expect("spec");
    fs::write(
        spec_root.join("compiler.md"),
        "# Compiler Contract\n\n## Ownership\n\nThe owner is shared.\n\n## Archivos esperados\n\n- crates/example/src/lib.rs\n",
    )
    .expect("compiler");
    fs::write(
        spec_root.join("acceptance.md"),
        "# Acceptance\n\n- Builds the graph.\n",
    )
    .expect("acceptance");
}

fn write_server_views_spec_fixture(root: &Path) {
    let spec_root = root.join("specs/features/00003-server-views-feature");
    fs::create_dir_all(&spec_root).expect("spec root");
    fs::write(
        spec_root.join("spec.md"),
        "# Server Views Feature\n\nStatus: Draft\n\n## Resumen\n\nAdd backend endpoints and views components.\n",
    )
    .expect("spec");
    fs::write(
        spec_root.join("server.md"),
        "# Server Contract\n\n## Endpoints\n\n- GET /api/example\n",
    )
    .expect("server");
    fs::write(
        spec_root.join("views.md"),
        "# Views Contract\n\n## Components\n\n- Button updates rendered pages.\n",
    )
    .expect("views");
    fs::write(
        spec_root.join("acceptance.md"),
        "# Acceptance\n\n- Serves the endpoint.\n- Renders the page.\n",
    )
    .expect("acceptance");
}
