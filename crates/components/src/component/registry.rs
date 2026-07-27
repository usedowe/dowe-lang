#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentRegistry;

impl ComponentRegistry {
    pub fn get(self, name: &str) -> Option<BuiltinComponent> {
        BuiltinComponent::from_name(name)
    }
}

pub const COMPONENT_REGISTRY: ComponentRegistry = ComponentRegistry;
