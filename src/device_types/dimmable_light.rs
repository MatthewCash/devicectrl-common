use super::define_state_structs;

define_state_structs!(DimmableLightState, {
    pub power: bool,
    pub brightness: u8,
});
