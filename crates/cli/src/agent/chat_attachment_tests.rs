#[cfg(test)]
mod attachment_tests {
    use super::*;

    #[test]
    fn attaches_readme_and_quoted_dropped_image() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("README.md"), "# local\n").unwrap();
        let image = root.path().join("design.png");
        std::fs::write(&image, b"image").unwrap();
        let mut options = AgentPrepareOptions::default();
        let prompt = attach_prompt_references(
            &format!("review @README.md and '{}'", image.display()),
            root.path(),
            &mut options,
        );
        assert!(prompt.contains("Untrusted local file context"));
        assert!(prompt.contains("# local"));
        assert_eq!(options.image_paths, vec![fs::canonicalize(image).unwrap()]);
    }

    #[test]
    fn leaves_email_like_text_and_missing_references_untouched() {
        let root = tempfile::tempdir().unwrap();
        let mut options = AgentPrepareOptions::default();
        let prompt =
            attach_prompt_references("email me at @name @missing.md", root.path(), &mut options);
        assert_eq!(prompt, "email me at @name @missing.md");
        assert!(options.image_paths.is_empty());
    }
}

