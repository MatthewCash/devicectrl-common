use super::define_state_structs;

define_state_structs!(ColorBulbState, {
    pub power: bool,
    pub brightness: u8,
    pub hue: u8,
    pub saturation: u8,
});
