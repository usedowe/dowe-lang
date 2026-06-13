fn responsive_bool(entries: &[(Breakpoint, bool)]) -> ResponsiveValue<bool> {
    ResponsiveValue::ordered(
        entries
            .iter()
            .map(|(breakpoint, value)| ResponsiveEntry {
                breakpoint: *breakpoint,
                value: *value,
            })
            .collect(),
    )
}
