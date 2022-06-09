#![cfg(target_arch = "wasm32")]

use smart_factory_environment::empty_environment::{
    EmptyEnvironmentSettings, InfiniteEmptyEnvironment,
};
use smart_factory_environment::environment::AgentEnvironment;
use smart_factory_wasm_port::sleep;
use std::time::Duration;
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
    InfiniteEmptyEnvironment::new(log_function, sleep);
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
    let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep);
    environment.run(&EmptyEnvironmentSettings { agent_count: 0 });
    assert_eq!(log_message, "Starting");
}

#[wasm_bindgen_test]
pub async fn when_halting_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep);
    let run = environment.run(&EmptyEnvironmentSettings { agent_count: 1 });
    let wait = Box::pin(sleep(Duration::from_secs(1)));
    futures::future::select(run, wait).await;
    environment.halt();
    environment.get_agents().iter().for_each(|agent| {
        assert!(agent.was_called);
    });
    assert_eq!(log_message, "Halting");
}
