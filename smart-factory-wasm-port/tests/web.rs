#![cfg(target_arch = "wasm32")]

use smart_factory_environment::empty_environment::{EmptyEnvironment, EmptyEnvironmentSettings};
use smart_factory_environment::environment::AgentEnvironment;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn when_creating_new_environment_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    EmptyEnvironment::new(&EmptyEnvironmentSettings {}, log_function);
    assert_eq!(log_message, "Creating new environment");
}

#[wasm_bindgen_test]
pub fn when_starting_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    let mut environment = EmptyEnvironment::new(&EmptyEnvironmentSettings {}, log_function);
    environment.run();
    assert_eq!(log_message, "Starting");
}

#[wasm_bindgen_test]
pub fn when_halting_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    let mut environment = EmptyEnvironment::new(&EmptyEnvironmentSettings {}, log_function);
    environment.halt();
    assert_eq!(log_message, "Halting");
}
