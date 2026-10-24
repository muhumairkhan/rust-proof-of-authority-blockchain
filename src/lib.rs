pub fn name() -> &'static str {
    "rust-poa-chain"
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_correct() {
        assert_eq!(name(), "rust-poa-chain");
    }
}
