pub mod agent;
pub mod empty_environment;
pub mod environment;
mod event;
mod event_queue;
pub mod message;

//FIXME: temporary function for testing purposes. Remove when smart-factory-server finally has relevant tests
pub fn greet_message(name: &str) -> String {
    format!("Hello, {}!", name.trim_end())
}
