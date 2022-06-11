#![cfg(target_arch = "wasm32")]

use smart_factory_environment::empty_environment::{
    EmptyEnvironmentSettings, InfiniteEmptyEnvironment,
};
use smart_factory_environment::environment::AgentEnvironment;
use smart_factory_environment::message::OutgoingQueueMessage;
use smart_factory_wasm_port::sleep;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const ITER_COUNT_SLEEP: u64 = 5000;
const SLEEP_DURATION_MS: u64 = 100;

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
    environment.run(EmptyEnvironmentSettings::new(
        0,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
    ));
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
    let run = environment.run(EmptyEnvironmentSettings::new(
        1,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
    ));
    let wait = Box::pin(sleep(Duration::from_secs(1)));
    futures::future::select(run, wait).await;
    environment.halt();
    environment.get_agents().iter().for_each(|agent| {
        assert!(agent.was_called);
    });
    assert_eq!(log_message, "Halting");
}

#[wasm_bindgen_test]
pub fn when_change_sleep_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep);
    environment.run(EmptyEnvironmentSettings::new(
        0,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
    ));
    environment.change_sleep_time(1000);
    assert_eq!(log_message, "Changing sleep time");
}

#[wasm_bindgen_test]
pub fn when_change_sleep_iter_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep);
    environment.run(EmptyEnvironmentSettings::new(
        0,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
    ));
    environment.change_sleep_iter_count(1000);
    assert_eq!(log_message, "Changing sleep iter count");
}

#[wasm_bindgen_test]
pub fn when_change_iter_then_call_log() {
    let mut log_message = String::new();
    let log_function = |message: &str| {
        println!("{}", message);
        log_message = message.to_string();
        smart_factory_wasm_port::log(message)
    };
    let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep);
    environment.run(EmptyEnvironmentSettings::new(
        0,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
    ));
    environment.change_max_iter_count(1000);
    assert_eq!(log_message, "Changing max iter count");
}

#[wasm_bindgen_test]
pub async fn it_runs() {
    let log_function = |message: &str| smart_factory_wasm_port::log(message);
    let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep);
    let result = environment.run(EmptyEnvironmentSettings::new(
        0,
        SLEEP_DURATION_MS,
        ITER_COUNT_SLEEP,
    ));
    let result = result.await;
    assert!(result.is_ok());
    assert!(environment.receiver.is_some());
    let receiver = environment.receiver.unwrap();
    let _result = receiver.try_recv();
    assert!(matches!(
        Result::<OutgoingQueueMessage, TryRecvError>::Ok(OutgoingQueueMessage::Started),
        _result
    ));
}
