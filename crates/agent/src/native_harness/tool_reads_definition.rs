fn definition(
    name: &str,
    description: &str,
    properties: Value,
    required: &[&str],
) -> AgentToolDefinition {
    AgentToolDefinition {
        tool_type: "function".into(),
        function: AgentToolFunction {
            name: name.into(),
            description: description.into(),
            parameters: json!({"type":"object","properties":properties,"required":required,"additionalProperties":false}),
        },
    }
}
