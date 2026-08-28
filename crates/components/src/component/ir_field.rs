include!(concat!(env!("OUT_DIR"), "/ir_field_types.rs"));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IrFieldPath {
    pub root: IrFieldRoot,
    pub fields: &'static [IrFieldSegment],
}

impl IrFieldPath {
    pub const fn new(root: IrFieldRoot, fields: &'static [IrFieldSegment]) -> Self {
        Self { root, fields }
    }

    pub fn as_string(self) -> String {
        let mut value = self.root.as_str().to_string();
        for field in self.fields {
            value.push('.');
            value.push_str(field.as_str());
        }
        value
    }

    pub const fn is_empty(self) -> bool {
        false
    }
}
