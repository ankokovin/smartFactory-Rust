use crate::environment::{AgentEnvironment, EnvironmentSettings};
use crate::event_engine::EventEngine;

pub struct EmptyEnvironmentSettings {}

impl EnvironmentSettings for EmptyEnvironmentSettings {}

pub struct EmptyEnvironment<'a, LogFunction>
where
    LogFunction: FnMut(&str),
{
    log: LogFunction,
    event_engine: EventEngine<'a>,
}

impl<'a, LogFunction> EmptyEnvironment<'a, LogFunction> where LogFunction: FnMut(&str) {}

impl<LogFunction> AgentEnvironment<LogFunction, EmptyEnvironmentSettings>
    for EmptyEnvironment<'_, LogFunction>
where
    LogFunction: FnMut(&str),
{
    fn new(_settings: &EmptyEnvironmentSettings, mut log: LogFunction) -> Self {
        log("Creating new environment");
        Self {
            log,
            event_engine: EventEngine::new(Default::default()),
        }
    }

    fn run(&mut self) {
        (self.log)("Starting");
        let _result = self.event_engine.start(Default::default());
    }

    fn halt(&mut self) {
        (self.log)("Halting");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::AgentEnvironment;

    #[test]
    pub fn when_creating_new_environment_then_call_log() {
        let mut log_message = String::new();
        let log_function = |message: &str| {
            println!("{}", message);
            log_message = message.to_string();
        };
        EmptyEnvironment::new(&EmptyEnvironmentSettings {}, log_function);
        assert_eq!(log_message, "Creating new environment");
    }

    #[test]
    pub fn when_starting_then_call_log() {
        let mut log_message = String::new();
        let log_function = |message: &str| {
            println!("{}", message);
            log_message = message.to_string();
        };
        let mut environment = EmptyEnvironment::new(&EmptyEnvironmentSettings {}, log_function);
        environment.run();
        assert_eq!(log_message, "Starting");
    }

    #[test]
    pub fn when_halting_then_call_log() {
        let mut log_message = String::new();
        let log_function = |message: &str| {
            println!("{}", message);
            log_message = message.to_string();
        };
        let mut environment = EmptyEnvironment::new(&EmptyEnvironmentSettings {}, log_function);
        environment.halt();
        assert_eq!(log_message, "Halting");
    }
}
