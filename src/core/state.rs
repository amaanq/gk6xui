use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use lazy_static::lazy_static;
use num::FromPrimitive;
use parking_lot::Mutex;
use serde::Deserialize;

use super::{
    key_values::{self, DriverValue},
    Layer,
};

lazy_static! {
    pub static ref STATES_BY_MODEL_ID: Mutex<HashMap<u32, State>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Default)]
pub struct State {
    firmware_id: u32,
    firmware_major_version: u8,
    firmware_minor_version: u8,
    firmware_version: u16,

    model_id: u32,
    model_name: String,

    layers: HashMap<Layer, StateLayer>,
    buffer_size_a: u8,
    buffer_size_b: u8,

    keys_by_location_code: HashMap<i32, Key>,
    keys_by_logic_code: HashMap<i32, Key>,
    keys_by_driver_value: HashMap<u32, Key>,
    /// A unique name for every key. Keys with duplicate DriverValue entries will be given seperate names here.
    keys_by_driver_value_name: HashMap<String, Key>,
}

/// Getters
impl State {
    pub fn get_firmware_version(&self) -> u16 {
        (self.firmware_major_version as u16) << 8 | self.firmware_minor_version as u16
    }

    pub fn firmware_version(&self) -> String {
        format!(
            "v{}.{}",
            self.firmware_major_version, self.firmware_minor_version
        )
    }

    pub fn get_max_logic_code(&self) -> i32 {
        self.buffer_size_a as i32 + self.buffer_size_b as i32
    }

    pub fn has_initialized_buffers(&self) -> bool {
        self.get_max_logic_code() > 0
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load() {
        //  string modelListPath = Path.Combine(Program.DataBasePath, "device", "modellist.json");

        #[derive(Deserialize)]
        struct DeviceModel {
            #[serde(rename = "modelID")]
            model_id: u32,
            #[serde(rename = "firmwareID")]
            firmware_id: String,
            name: String,
            #[serde(rename = "LEType")]
            le_type: String,
        }

        let model_list_path = "assets/device/models.json";
        // serde deserialize
        let models = serde_json::from_reader::<_, Vec<DeviceModel>>(BufReader::new(
            File::open(model_list_path).unwrap(),
        ))
        .unwrap();

        for model in models {
            let mut keyboard_state = State::new();
            keyboard_state.model_id = model.model_id;
            // parse firmware_id as "0x1234" to u32
            keyboard_state.firmware_id = u32::from_str_radix(&model.firmware_id, 16).unwrap();
            keyboard_state.model_name = model.name;
            keyboard_state.load_keys();
            STATES_BY_MODEL_ID
                .lock()
                .insert(model.model_id, keyboard_state);
        }
    }

    pub fn get_keyboard_state(model_id: u32) -> State {
        STATES_BY_MODEL_ID.lock().get(&model_id).unwrap().clone()
    }

    pub fn initialize_buffers(&mut self, size_a: u8, size_b: u8) {
        self.layers.clear();

        self.buffer_size_a = 0;
        self.buffer_size_b = 0;

        if self.firmware_id == 0 || self.model_id == 0 {
            return;
        }

        self.buffer_size_a = size_a;
        self.buffer_size_b = size_b;

        self.create_factory_default_layers();
    }

    pub fn create_factory_default_layers(&mut self) {
        self.create_layer(Layer::Base, None);
        self.create_layer(Layer::Driver, None);
        self.create_layer(Layer::Layer1, None);
        self.create_layer(Layer::Layer2, None);
        self.create_layer(Layer::Layer3, None);
    }

    pub fn create_layer(
        &mut self,
        layer: Layer,
        model_data: Option<HashMap<String, serde_json::Value>>,
    ) {
        self.layers.insert(layer, StateLayer::new());
        let state = self.layers.get_mut(&layer).unwrap();

        let mut profile_layer_name = "";

        match layer {
            Layer::Driver => profile_layer_name = "_online_1",
            Layer::Layer1 => profile_layer_name = "_offline_1",
            Layer::Layer2 => profile_layer_name = "_offline_2",
            Layer::Layer3 => profile_layer_name = "_offline_3",
            _ => {}
        }

        let profile_path = format!(
            "assets/device/{}/data/profile{}.json",
            self.model_id, profile_layer_name
        );

        if Path::new(&profile_path).exists() {
            let profile = serde_json::from_reader::<_, HashMap<String, serde_json::Value>>(
                BufReader::new(File::open(profile_path).unwrap()),
            )
            .unwrap();
            state.factory_default_model_data = Some(profile);
        }

        let model_data = if let Some(model_data) = model_data {
            model_data
        } else {
            if let Some(ref factory_default_model_data) = state.factory_default_model_data {
                factory_default_model_data.clone()
            } else {
                return;
            }
        };

        let mut key_set = vec![
            key_values::UNUSED_KEY_VALUE;
            self.buffer_size_a as usize * self.buffer_size_b as usize
        ];
        let mut fn_key_set = vec![
            key_values::UNUSED_KEY_VALUE;
            self.buffer_size_a as usize * self.buffer_size_b as usize
        ];

        state.key_press_lighting_effect = vec![0xFF; 128];

        std::mem::drop(state);

        self.setup_driver_key_set_buffer(&model_data, "KeySet", &mut key_set);
        self.setup_driver_key_set_buffer(&model_data, "FnKeySet", &mut fn_key_set);

        let state = self.layers.get_mut(&layer).unwrap();

        state.key_set = key_set;
        state.fn_key_set = fn_key_set;

        if let Some(device_le_data) = model_data.get("DeviceLE") {
            if let Some(_) = device_le_data.get("LESet") {
                state.has_le_set = true;
            }
        }
    }

    fn setup_driver_key_set_buffer(
        &mut self,
        model_data: &HashMap<String, serde_json::Value>,
        key: &str,
        driver_key_set_array: &mut Vec<u32>,
    ) {
        if let Some(key_set_data) = model_data.get(key) {
            for key_set_obj in key_set_data.as_array().unwrap() {
                let key_set = key_set_obj.as_object().unwrap();
                let index = key_set.get("Index").unwrap().as_i64().unwrap();
                let mut driver_value =
                    i32::from_str_radix(key_set.get("DriverValue").unwrap().as_str().unwrap(), 16)
                        .unwrap();
                if (index as usize) < driver_key_set_array.len() {
                    if driver_value == 0xA030001 || driver_value == 0xa010001 {
                        driver_value += index as i32;
                    }
                }
                driver_key_set_array[index as usize] = driver_value as u32;
            }
        }
    }

    fn get_default_profile_driver_values(&mut self) -> Option<Vec<u32>> {
        let profile_path = format!("assets/device/{}/data/profile.json", self.model_id);
        if Path::new(&profile_path).exists() {
            let model_data = serde_json::from_reader::<_, HashMap<String, serde_json::Value>>(
                BufReader::new(File::open(profile_path).unwrap()),
            )
            .unwrap();
            let mut result = vec![key_values::UNUSED_KEY_VALUE; i16::MAX as usize];
            self.setup_driver_key_set_buffer(&model_data, "KeySet", &mut result);
            return Some(result);
        }
        return None;
    }

    fn load_keys(&mut self) {
        self.keys_by_location_code.clear();
        self.keys_by_logic_code.clear();
        self.keys_by_driver_value.clear();
        self.keys_by_driver_value_name.clear();

        let driver_values = self.get_default_profile_driver_values().unwrap();

        let keys_path = format!("assets/device/{}/data/keymap.js", self.model_id);
        if !Path::new(&keys_path).exists() {
            return;
        }

        let text = fs::read_to_string(&keys_path).unwrap();
        if text.replace(' ', "").contains("KeyName:") {
            log::debug!("TODO: Handle keymap which is declared as JS `{keys_path}`");
            return;
        }

        let mut device_keys = serde_json::from_str::<Vec<Key>>(&text).unwrap();
        for key in device_keys.iter_mut() {
            if key.logic_code >= 0
                && driver_values[key.logic_code as usize] != key_values::UNUSED_KEY_VALUE
            {
                key.driver_value = driver_values[key.logic_code as usize];
            } else {
                if key.logic_code > 0 {
                    if let Some(all_keys_key) = key_values::KEYS_BY_LOGIC_CODE.get(&key.logic_code)
                    {
                        key.driver_value = all_keys_key.driver_value;
                    } else {
                        log::debug!("Couldn't find DriverValue for key `{}` logicCode: {} locationCode: {} modelId: {} modelName: {}", key.key_name, key.logic_code, key.location_code, self.model_id, self.model_name);
                    }
                } else {
                    key.driver_value = key_values::UNUSED_KEY_VALUE;
                }
            }

            self.keys_by_location_code
                .insert(key.location_code, key.clone());
            self.keys_by_logic_code.insert(key.logic_code, key.clone());
            self.keys_by_driver_value
                .insert(key.driver_value, key.clone());

            for i in 1..i32::MAX {
                let driver_value_name = format!(
                    "{}{}",
                    DriverValue::from_u32(key.driver_value).unwrap(),
                    if i > 1 {
                        format!("_{}", i)
                    } else {
                        "".to_string()
                    }
                );
                if !self
                    .keys_by_driver_value_name
                    .contains_key(&driver_value_name)
                {
                    key.driver_value_name = driver_value_name.clone();
                    self.keys_by_driver_value_name
                        .insert(driver_value_name, key.clone());
                    break;
                }
            }
        }
    }

    pub fn get_layer(&self, layer: Layer) -> Option<&StateLayer> {
        self.layers.get(&layer)
    }

    pub fn get_key_at_location_code(&self, location_code: i32) -> Option<&Key> {
        self.keys_by_location_code.get(&location_code)
    }

    pub fn get_key_by_logic_code(&self, logic_code: i32) -> Option<&Key> {
        self.keys_by_logic_code.get(&logic_code)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Key {
    pub key_name: String,
    pub show: String,
    pub logic_code: i32,
    pub location_code: i32,
    pub position: KeyRect,
    pub driver_value: u32,

    /// Unique for a given key, even if there are keys with duplicate driver values
    pub driver_value_name: String,
}

impl Key {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Default, Copy, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct KeyRect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Default)]
pub struct StateLayer {
    pub factory_default_model_data: Option<HashMap<String, serde_json::Value>>,
    pub key_set: Vec<u32>,
    pub fn_key_set: Vec<u32>,
    pub key_press_lighting_effect: Vec<u8>,

    /// Only used for "driver" layer? (17 01) see [`OpCodes::DriverLayerSetConfig`]
    pub has_le_set: bool,
}

impl StateLayer {
    pub fn new() -> Self {
        Self::default()
    }
}
