use core::ops::{Add, Mul, Sub};
use num_traits::{NumCast, cast};
use serde_derive::{Deserialize, Serialize};

use crate::device_types::{NumericState, ceiling_fan::FanDirection, switch::SwitchPower};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum NumericUpdate<T: Copy = u32, F: Copy = f32> {
    Percent(F),
    Absolute(T),
    DeltaAbsolute(T), // add value
    DeltaPercent(F),  // add percentage
    ScaleBy(F),       // multiply by value
}

// Helper function to apply a NumericUpdate to a current value
impl<T, F> NumericUpdate<T, F>
where
    T: Copy + Default + Ord + NumCast + Add<Output = T> + Sub<Output = T>,
    F: Copy + Default + NumCast + Mul<Output = F>,
{
    pub fn apply_to(&self, state: &NumericState<T>) -> T {
        // clamp but without the assert that min <= max
        fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
            if input < min {
                min
            } else if input > max {
                max
            } else {
                input
            }
        }

        // infallible cast with default fallback
        fn safe_cast<A: NumCast, B: NumCast + Default>(value: A) -> B {
            cast::<A, B>(value).unwrap_or_default()
        }

        clamp(
            match self {
                NumericUpdate::Percent(pct) => {
                    state.min + safe_cast(*pct * safe_cast(state.max - state.min))
                }
                NumericUpdate::Absolute(new_value) => *new_value,
                NumericUpdate::DeltaAbsolute(delta) => state.value + *delta,
                NumericUpdate::DeltaPercent(pct) => {
                    state.value + safe_cast(*pct * safe_cast(state.value - state.min))
                }
                NumericUpdate::ScaleBy(factor) => safe_cast(*factor * safe_cast(state.value)),
            },
            state.min,
            state.max,
        )
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AttributeUpdate {
    Power(SwitchPower),
    Brightness(NumericUpdate),
    ColorTemp(NumericUpdate),
    Hue(NumericUpdate<u16>),
    Saturation(NumericUpdate),
    FanSpeed(NumericUpdate),
    FanDirection(FanDirection),
}
