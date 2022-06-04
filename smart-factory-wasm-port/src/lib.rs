extern crate wasm_bindgen;

use smart_factory_environment;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    smart_factory_environment::greet_message(name)
}