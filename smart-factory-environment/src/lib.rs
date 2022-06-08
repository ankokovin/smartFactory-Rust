pub mod empty_environment;
pub mod environment;

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
