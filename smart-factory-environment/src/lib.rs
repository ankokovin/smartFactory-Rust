pub mod agent;
pub mod empty_environment;
pub mod environment;
mod event;
mod event_queue;
pub mod message;

pub fn greet_message(name: &str) -> String {
    format!("Hello, {}!", name.trim_end())
}

#[cfg(test)]
mod tests {
    use crate::greet_message;

    #[test]
    fn it_works() {
        assert_eq!("Hello, World!", greet_message("World"));
    }
}
