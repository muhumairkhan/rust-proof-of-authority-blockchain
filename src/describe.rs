use crate::account::Account;

pub trait Describe {
    fn describe(&self) -> String;
}

impl Describe for Account {
    fn describe(&self) -> String {
        format!("balance={} nonce={}", self.balance, self.nonce)
    }
}

pub fn print_description<T: Describe>(item: &T) {
    println!("{}", item.describe());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_formats_account() {
        assert_eq!(Account::new(42).describe(), "balance=42 nonce=0");
    }
}
