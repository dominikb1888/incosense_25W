pub fn hello() -> String {
    "Hello, World!".to_string()
}

fn main() {
    println!("{}", hello());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hello_returns_correct_string() {
        assert_eq!(hello(), "Hello, World!");
    }
}

