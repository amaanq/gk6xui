use std::{collections::HashMap, sync::{Arc, atomic::AtomicBool}};

use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

use super::keyboard::Keyboard;

lazy_static! {
    pub static ref PRODUCTS: OnceCell<HashMap<u16, Vec<u16>>> = OnceCell::new();
    pub static ref CONNECTED_DEVICES: Arc<Mutex<HashMap<String, Keyboard>>> =
        Arc::new(Mutex::new(HashMap::new()));
}
pub static mut IS_LISTENING: AtomicBool = AtomicBool::new(false);

pub fn init() {
    // populate DEVICES

    let mut products = HashMap::new();
    products.insert(0x1EA7, vec![0x907]);
    PRODUCTS.set(products).unwrap();
}

pub async fn start_listening() {
    let _devices = CONNECTED_DEVICES.lock().await;
    // for (path, device) in devices.iter_mut() {
    //     device.start_listening(path).await;
    // }
}
