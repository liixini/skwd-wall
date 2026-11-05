mod action;
mod map;
mod nav_keys;
mod trigger;

pub use action::{ActiveScopes, InputAction};
pub use map::InputMap;
pub use nav_keys::NavKeys;
pub use trigger::{
    KeyId, KeySpec, Mods, MouseButton, MouseSpec, SLOTS, Trigger, binding_config, binding_label,
    parse_binding, slot_triggers, with_slot, without,
};

#[cfg(test)]
mod tests;
