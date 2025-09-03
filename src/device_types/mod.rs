pub mod ceiling_fan;
pub mod color_light;
pub mod dimmable_light;
pub mod switch;

macro_rules! define_state_structs {
    (
        $name:ident,
        $properties:tt
    ) => {
        paste::paste! {
            #[derive(Clone, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct $name $properties

            #[discretionary::make_optional]
            #[derive(Clone, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct [<$name Update>] $properties
        }
    };
}

pub(crate) use define_state_structs;
