use serde_derive::{Deserialize, Serialize};

pub mod ceiling_fan;
pub mod color_light;
pub mod dimmable_light;
pub mod switch;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumericState<T: Copy = u32> {
    pub value: T,
    pub min: T,
    pub max: T,
    pub step: T,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumericProperties<T: Copy = u32> {
    pub min: T,
    pub max: T,
    pub step: T,
}

impl<T: Copy> NumericProperties<T> {
    pub fn to_state(&self, value: T) -> NumericState<T> {
        NumericState {
            value,
            min: self.min,
            max: self.max,
            step: self.step,
        }
    }
}
