pub fn register_consumed_prop(
    registry: &mut dowe_components::PropConsumptionRegistry,
    component: dowe_components::BuiltinComponent,
    prop: impl Into<String>,
    ir_field: impl Into<String>,
) {
    dowe_components::register_consumed_prop(registry, component, prop, ir_field);
}
