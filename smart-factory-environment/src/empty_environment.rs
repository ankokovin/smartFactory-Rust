use crate::environment::{AgentEnvironment, EnvironmentSettings};

pub struct EmptyEnvironmentSettings {}

impl EnvironmentSettings for EmptyEnvironmentSettings {}

pub struct EmptyEnvironment<LogFunction>
where
    LogFunction: FnMut(&str),
{
    log: LogFunction,
}

impl<LogFunction> EmptyEnvironment<LogFunction> where LogFunction: FnMut(&str) {}

impl<LogFunction> AgentEnvironment<LogFunction, EmptyEnvironmentSettings>
    for EmptyEnvironment<LogFunction>
where
    LogFunction: FnMut(&str),
{
    fn new(_settings: &EmptyEnvironmentSettings, mut log: LogFunction) -> Self {
        log("Creating new environment");
        Self { log }
    }

    fn run(&mut self) {
        (self.log)("Starting");
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
