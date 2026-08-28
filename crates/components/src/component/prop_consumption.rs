#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedProp {
    pub component: BuiltinComponent,
    pub item: Option<ViewItemKind>,
    pub prop: String,
    pub ir_field: String,
}

#[derive(Debug, Default)]
pub struct PropConsumptionRegistry {
    entries: Vec<ConsumedProp>,
}

impl PropConsumptionRegistry {
    pub fn register_consumed_prop(
        &mut self,
        component: BuiltinComponent,
        prop: impl Into<String>,
        ir_field: impl Into<String>,
    ) {
        let entry = ConsumedProp {
            component,
            item: None,
            prop: prop.into(),
            ir_field: ir_field.into(),
        };
        if !self.entries.contains(&entry) {
            self.entries.push(entry);
        }
        debug_assert!(self.validate().is_ok(), "invalid consumed prop registry: {:?}", self.entries);
    }

    pub fn register_consumed_item(
        &mut self,
        component: BuiltinComponent,
        item: ViewItemKind,
        prop: impl Into<String>,
        ir_field: impl Into<String>,
    ) {
        let entry = ConsumedProp {
            component,
            item: Some(item),
            prop: prop.into(),
            ir_field: ir_field.into(),
        };
        if !self.entries.contains(&entry) {
            self.entries.push(entry);
        }
        debug_assert!(self.validate().is_ok(), "invalid consumed prop registry: {:?}", self.entries);
    }

    pub fn entries(&self) -> &[ConsumedProp] {
        &self.entries
    }

    pub fn validate(&self) -> ComponentResult<()> {
        for entry in &self.entries {
            let Some(definition) = VIEW_PROP_INVENTORY
                .iter()
                .filter(|definition| definition.prop == entry.prop)
                .find(|definition| match definition.owner {
                    ViewPropOwner::Component(owner) => entry.item.is_none() && owner == entry.component,
                    ViewPropOwner::Item(item) => entry.item.is_some_and(|value| value == item),
                    ViewPropOwner::CommonStyle => false,
                })
                .or_else(|| {
                    VIEW_PROP_INVENTORY.iter().find(|definition| {
                        definition.prop == entry.prop
                            && matches!(definition.owner, ViewPropOwner::CommonStyle)
                    })
                })
            else {
                return Err(ComponentError::unknown_prop(
                    entry.component,
                    &entry.prop,
                ));
            };
            if definition.ir_field.as_string() != entry.ir_field {
                return Err(ComponentError::invalid_prop(
                    &entry.prop,
                    "registered IR field does not match the inventory",
                ));
            }
        }
        Ok(())
    }
}

pub fn register_consumed_prop(
    registry: &mut PropConsumptionRegistry,
    component: BuiltinComponent,
    prop: impl Into<String>,
    ir_field: impl Into<String>,
) {
    registry.register_consumed_prop(component, prop, ir_field);
}

pub fn register_consumed_item(
    registry: &mut PropConsumptionRegistry,
    component: BuiltinComponent,
    item: ViewItemKind,
    prop: impl Into<String>,
    ir_field: impl Into<String>,
) {
    registry.register_consumed_item(component, item, prop, ir_field);
}
