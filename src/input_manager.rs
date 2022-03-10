use std::sync::Arc;

use dashmap::DashMap;
use glam::Vec2;
use parking_lot::RwLock;
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton, VirtualKeyCode},
};

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PressState {
    None,
    Pressed,
    Held,
    Released,
}

lazy_static! {
    static ref PREVIOUS_INPUT_MAP: Arc<DashMap<VirtualKeyCode, PressState>> =
        Arc::new(DashMap::default());
    static ref PREVIOUS_MOUSE_MAP: Arc<DashMap<MouseButton, PressState>> =
        Arc::new(DashMap::default());
    static ref INPUT_MAP: Arc<DashMap<VirtualKeyCode, PressState>> = Arc::new(DashMap::default());
    static ref MOUSE_MAP: Arc<DashMap<MouseButton, PressState>> = Arc::new(DashMap::default());
    static ref MOUSE_DELTA: Arc<RwLock<PhysicalPosition<f64>>> =
        Arc::new(RwLock::new(PhysicalPosition::new(0.0, 0.0)));
    static ref PREVIOUS_MOUSE_POS: Arc<RwLock<PhysicalPosition<f64>>> =
        Arc::new(RwLock::new(PhysicalPosition::new(0.0, 0.0)));
    static ref MOUSE_POS: Arc<RwLock<PhysicalPosition<f64>>> =
        Arc::new(RwLock::new(PhysicalPosition::new(0.0, 0.0)));
}

pub fn update_inputs() {
    // Pressed -> Held
    INPUT_MAP.iter_mut().for_each(|mut key| {
        if *key.value() == PressState::Pressed
            && PREVIOUS_INPUT_MAP
                .get(key.key())
                .map_or(false, |previous| *previous.value() == PressState::Pressed)
        {
            *key.value_mut() = PressState::Held;
        }
    });

    MOUSE_MAP.iter_mut().for_each(|mut button| {
        if *button.value() == PressState::Pressed
            && PREVIOUS_MOUSE_MAP
                .get(button.key())
                .map_or(false, |previous| *previous.value() == PressState::Pressed)
        {
            *button.value_mut() = PressState::Held;
        }
    });

    // Released -> None
    INPUT_MAP.iter_mut().for_each(|mut key| {
        if *key.value() == PressState::Released
            && PREVIOUS_INPUT_MAP
                .get(key.key())
                .map_or(false, |previous| *previous.value() == PressState::Released)
        {
            *key.value_mut() = PressState::None;
        }
    });

    MOUSE_MAP.iter_mut().for_each(|mut button| {
        if *button.value() == PressState::Released
            && PREVIOUS_MOUSE_MAP
                .get(button.key())
                .map_or(false, |previous| *previous.value() == PressState::Released)
        {
            *button.value_mut() = PressState::None;
        }
    });

    // Update previous map
    INPUT_MAP.iter().for_each(|key| {
        PREVIOUS_INPUT_MAP.insert(*key.key(), *key.value());
    });

    MOUSE_MAP.iter().for_each(|button| {
        PREVIOUS_MOUSE_MAP.insert(*button.key(), *button.value());
    });

    let mut mouse_delta_lock = MOUSE_DELTA.write();
    let mouse_pos_lock = MOUSE_POS.read();
    let mut previous_mouse_pos_lock = PREVIOUS_MOUSE_POS.write();
    (mouse_delta_lock.x, mouse_delta_lock.y) = (
        mouse_pos_lock.x - previous_mouse_pos_lock.x,
        mouse_pos_lock.y - previous_mouse_pos_lock.y,
    );
    (previous_mouse_pos_lock.x, previous_mouse_pos_lock.y) = (mouse_pos_lock.x, mouse_pos_lock.y);
}

pub fn set_key(key: VirtualKeyCode, state: PressState) {
    INPUT_MAP.insert(key, state);
}

pub fn get_key_down(key: VirtualKeyCode) -> bool {
    INPUT_MAP
        .get(&key)
        .map_or(false, |state| *state.value() == PressState::Pressed)
}

pub fn get_key_held(key: VirtualKeyCode) -> bool {
    INPUT_MAP
        .get(&key)
        .map_or(false, |state| *state.value() == PressState::Held)
}

pub fn get_key(key: VirtualKeyCode) -> bool {
    INPUT_MAP.get(&key).map_or(false, |state| {
        *state.value() == PressState::Held || *state.value() == PressState::Pressed
    })
}

pub fn get_key_up(key: VirtualKeyCode) -> bool {
    INPUT_MAP
        .get(&key)
        .map_or(false, |state| *state.value() == PressState::Released)
}

pub fn get_button_down(button: MouseButton) -> bool {
    MOUSE_MAP
        .get(&button)
        .map_or(false, |state| *state.value() == PressState::Pressed)
}

pub fn get_button_held(button: MouseButton) -> bool {
    MOUSE_MAP
        .get(&button)
        .map_or(false, |state| *state.value() == PressState::Held)
}

pub fn get_button(button: MouseButton) -> bool {
    MOUSE_MAP.get(&button).map_or(false, |state| {
        *state.value() == PressState::Held || *state.value() == PressState::Pressed
    })
}

pub fn get_button_up(button: MouseButton) -> bool {
    MOUSE_MAP
        .get(&button)
        .map_or(false, |state| *state.value() == PressState::Released)
}

pub fn get_mouse_delta() -> Vec2 {
    let lock = MOUSE_DELTA.read();
    Vec2::new(lock.x as f32, lock.y as f32)
}

pub fn set_mouse_button(button: &MouseButton, state: PressState) {
    MOUSE_MAP.insert(*button, state);
}

pub fn set_mouse_pos(pos: &PhysicalPosition<f64>) {
    let mut lock = MOUSE_POS.write();
    (lock.x, lock.y) = (pos.x, pos.y);
}
