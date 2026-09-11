#[cfg(test)]
mod image_generation_contract_tests {
    use super::*;

    #[test]
    fn schema_does_not_advertise_overwrite_and_reference_rejection_is_explicit() {
        let definition =
            HarnessTools::definitions_for_capabilities(HarnessRole::Execute, false, true)
                .into_iter()
                .find(|tool| tool.function.name == "generate_image")
                .expect("generate_image definition");
        let properties = &definition.function.parameters["properties"];
        assert!(properties.get("overwrite").is_none());
        assert!(properties.get("reference_image_path").is_some());
        assert!(
            definition
                .function
                .description
                .contains("reference_image_path is unsupported in v1")
        );

        let root = tempfile::tempdir().expect("temporary project root");
        let mut tools = HarnessTools::new(root.path(), "image-contract", HarnessConfig::default())
            .expect("harness tools");
        let error = tools
            .prepare(
                &ToolCall::new(
                    "reference",
                    "generate_image",
                    json!({
                        "prompt": "a textured background",
                        "destination": "assets/background.png",
                        "reason": "test unsupported reference",
                        "reference_image_path": "assets/reference.png"
                    }),
                ),
                HarnessRole::Execute,
            )
            .expect_err("reference images must remain rejected");
        assert!(
            error
                .to_string()
                .contains("reference_image_path is not supported")
        );
    }
}

#[cfg(test)]
mod skill_tool_definition_contract_tests {
    use super::*;

    #[test]
    fn mutating_tools_require_skill_id_not_hash() {
        let definitions =
            HarnessTools::definitions_for_capabilities(HarnessRole::Execute, false, false);
        let get_skill = definitions
            .iter()
            .find(|tool| tool.function.name == "get_skill")
            .expect("get_skill definition");
        assert!(get_skill.function.description.contains("logical skill id"));
        assert!(
            get_skill
                .function
                .description
                .contains("integrity metadata only")
        );

        for name in ["write_file", "write_asset", "edit_file"] {
            let definition = definitions
                .iter()
                .find(|tool| tool.function.name == name)
                .expect("mutating tool definition");
            let skill = &definition.function.parameters["properties"]["skill"];
            assert_eq!(skill["type"], "string");
            assert!(
                skill["description"]
                    .as_str()
                    .expect("skill description")
                    .contains("never use the hash")
            );
        }
    }
}

#[cfg(test)]
mod codegraph_contract_tests {
    use super::*;

    #[test]
    fn codegraph_advertises_only_bounded_read_tools() {
        let names = HarnessTools::definitions_for_capabilities(HarnessRole::Codegraph, true, true)
            .into_iter()
            .map(|tool| tool.function.name)
            .collect::<Vec<_>>();
        assert!(names.iter().all(|name| matches!(
            name.as_str(),
            "ask_user" | "get_skill" | "read_file" | "list_files" | "search" | "convert_svg"
        )));
    }

    #[test]
    fn codegraph_rejects_mutating_and_execution_calls() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "codegraph", HarnessConfig::default()).unwrap();
        for name in [
            "write_file",
            "edit_file",
            "write_asset",
            "propose_instruction_update",
            "shell",
            "generate_image",
            "capture_web_screenshot",
        ] {
            let call = ToolCall::new("codegraph", name, json!({}));
            assert!(
                tools.prepare(&call, HarnessRole::Codegraph).is_err(),
                "accepted {name}"
            );
        }
    }
}

#[cfg(test)]
mod svg_conversion_tests {
    use super::*;

    #[test]
    fn advertises_svg_conversion_as_a_bounded_read_tool() {
        let definitions =
            HarnessTools::definitions_for_capabilities(HarnessRole::Codegraph, false, false);
        let definition = definitions
            .iter()
            .find(|tool| tool.function.name == "convert_svg")
            .expect("convert_svg definition");
        let properties = &definition.function.parameters["properties"];

        assert_eq!(definition.function.parameters["required"], json!(["path"]));
        assert_eq!(properties["colors"]["default"], "original");
        assert_eq!(properties["format"]["default"], "source");
        assert_eq!(
            properties["colors"]["enum"],
            json!(["original", "tokens"])
        );
        assert_eq!(
            properties["format"]["enum"],
            json!(["source", "data"])
        );
        assert!(definition.function.description.contains("read-only"));
    }

    #[test]
    fn converts_source_and_runtime_data_without_creating_an_approval() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("assets")).unwrap();
        fs::write(
            root.path().join("assets/mark.svg"),
            r##"<svg viewBox="0 0 24 24"><path d="M4 4L20 4L12 20Z" fill="#2457D6"/></svg>"##,
        )
        .unwrap();
        let tools = HarnessTools::new(root.path(), "svg", HarnessConfig::default()).unwrap();

        let source = tools
            .execute_read(&ToolCall::new(
                "svg-source",
                "convert_svg",
                json!({"path":"assets/mark.svg"}),
            ))
            .unwrap();
        assert_eq!(source["status"], "converted");
        assert_eq!(source["colors"], "original");
        assert_eq!(source["format"], "source");
        assert!(source["content"].as_str().unwrap().contains("Svg viewBox"));
        assert!(source["content"].as_str().unwrap().contains("fill:\"#2457d6\""));

        let token_source = tools
            .execute_read(&ToolCall::new(
                "svg-tokens",
                "convert_svg",
                json!({"path":"assets/mark.svg","colors":"tokens","format":"source"}),
            ))
            .unwrap();
        assert!(token_source["content"].as_str().unwrap().contains("fill:\"primary\""));

        let data = tools
            .execute_read(&ToolCall::new(
                "svg-data",
                "convert_svg",
                json!({"path":"assets/mark.svg","format":"data"}),
            ))
            .unwrap();
        let data_content = data["content"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(data_content).unwrap();
        assert_eq!(parsed["viewBox"], "0 0 24 24");
        assert_eq!(parsed["paths"].as_array().unwrap().len(), 1);
        assert!(tools.pending.is_empty());
    }

    #[test]
    fn rejects_unsafe_paths_invalid_options_and_tokenized_runtime_data() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("mark.svg"), "<svg></svg>").unwrap();
        let tools = HarnessTools::new(root.path(), "svg-errors", HarnessConfig::default()).unwrap();

        let cases = [
            (json!({"path":"mark.txt"}), ".svg extension"),
            (json!({"path":"mark.svg","colors":"other"}), "colors"),
            (json!({"path":"mark.svg","format":"other"}), "format"),
            (json!({"path":"mark.svg","colors":"tokens","format":"data"}), "requires colors original"),
            (json!({"path":".dowe/mark.svg"}), "private, generated"),
            (json!({"path":"agents/private.svg"}), "private, generated"),
        ];
        for (arguments, message) in cases {
            let error = tools
                .execute_read(&ToolCall::new("svg-error", "convert_svg", arguments))
                .unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
        }
    }
}

#[cfg(test)]
mod list_files_tests {
    use super::*;

    #[test]
    fn missing_directory_returns_empty_successful_result() {
        let root = tempfile::tempdir().unwrap();
        let tools = HarnessTools::new(root.path(), "list-files", HarnessConfig::default()).unwrap();
        let result = tools
            .execute_read(&ToolCall::new(
                "list",
                "list_files",
                json!({"path": "assets", "offset": 7}),
            ))
            .unwrap();

        assert_eq!(
            result,
            json!({
                "exists": false,
                "paths": [],
                "next_offset": 0,
                "truncated": false,
            })
        );
    }

    #[test]
    fn existing_non_directory_returns_an_error() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("assets"), "not a directory").unwrap();
        let tools = HarnessTools::new(root.path(), "list-files", HarnessConfig::default()).unwrap();

        assert!(
            tools
                .execute_read(&ToolCall::new(
                    "list",
                    "list_files",
                    json!({"path": "assets"}),
                ))
                .is_err()
        );
    }
}
