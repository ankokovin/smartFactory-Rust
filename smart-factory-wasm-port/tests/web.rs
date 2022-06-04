#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn it_works() {
    let name = "World";

    let result = smart_factory_wasm_port::greet(name);

    assert_eq!("Hello, World!", result);
}