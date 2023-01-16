use hidapi::HidDevice;

use super::State;

pub struct Keyboard {
    device: HidDevice,
    state: State,
}
