#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleValue(pub u16);

impl ScaleValue {
    pub const fn from_half_steps(value: u16) -> Self {
        Self(value)
    }

    pub fn class_suffix(self) -> String {
        if self.0 % 2 == 0 {
            (self.0 / 2).to_string()
        } else {
            format!("{}.5", self.0 / 2)
        }
    }

    pub fn native_units(self) -> u16 {
        self.0 * 2
    }
}
