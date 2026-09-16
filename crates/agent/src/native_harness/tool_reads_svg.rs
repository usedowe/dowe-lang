impl HarnessTools {
    fn convert_svg(&self, args: &SvgConversionArgs) -> AgentResult<Value> {
        let original_colors = match args.colors.as_str() {
            "original" => true,
            "tokens" => false,
            _ => {
                return Err(AgentError::new(
                    "convert_svg colors must be original or tokens",
                ));
            }
        };
        match args.format.as_str() {
            "source" => {}
            "data" if original_colors => {}
            "data" => {
                return Err(AgentError::new(
                    "convert_svg format data requires colors original",
                ));
            }
            _ => return Err(AgentError::new("convert_svg format must be source or data")),
        }
        if args.path == "agents" || args.path.starts_with("agents/") {
            return Err(AgentError::new(
                "private, generated or instruction path is not application context",
            ));
        }
        let path = self.path(&args.path)?;
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
        {
            return Err(AgentError::new(
                "convert_svg path must have a .svg extension",
            ));
        }
        let metadata = fs::metadata(&path)?;
        if !metadata.is_file() {
            return Err(AgentError::new("convert_svg path must be a file"));
        }
        if metadata.len() > 262_144 {
            return Err(AgentError::new("convert_svg input exceeds 262144 bytes"));
        }
        let source = fs::read_to_string(&path)?;
        let content = match args.format.as_str() {
            "source" => dowe_stdlib::convert_svg(&source, original_colors)
                .map_err(|error| AgentError::new(error.to_string()))?,
            "data" => dowe_stdlib::convert_svg_data(&source)
                .map_err(|error| AgentError::new(error.to_string()))?
                .as_str()
                .ok_or_else(|| AgentError::new("convert_svg data result was not text"))?
                .to_string(),
            _ => unreachable!("format was validated above"),
        };
        let result = json!({
            "status": "converted",
            "path": args.path,
            "colors": args.colors,
            "format": args.format,
            "content": self.redactor.text(&content),
        });
        if serde_json::to_vec(&result)?.len() > self.config.max_output_bytes {
            return Err(AgentError::new(
                "convert_svg output exceeds the harness output budget; use a smaller SVG",
            ));
        }
        Ok(result)
    }
}
