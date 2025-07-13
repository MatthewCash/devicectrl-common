use super::define_state_structs;

define_state_structs!(LedStripState, {
    pub power: bool,
    pub brightness: u8,
});
