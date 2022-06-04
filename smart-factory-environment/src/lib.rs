pub fn greet_message(name: &str) -> String {
    format!("Hello, {}!", name.trim_end())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
