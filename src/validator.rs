/// The ordered list of validators. Order matters: it is the rotation order.
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    validators: Vec<String>,
}

impl ValidatorSet {
    /// Returns None if the list is empty (a chain with no validators makes no sense).
    pub fn new(validators: Vec<String>) -> Option<ValidatorSet> {
        if validators.is_empty() {
            return None;
        }
        Some(ValidatorSet { validators })
    }

    /// Which validator owns a given slot: slot % number_of_validators.
    pub fn validator_for_slot(&self, slot: u64) -> &str {
        let index = (slot % self.validators.len() as u64) as usize;
        &self.validators[index]
    }

    pub fn all(&self) -> &[String] {
        &self.validators
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validator_set_rejects_empty_list() {
        assert!(ValidatorSet::new(Vec::new()).is_none());
    }

    #[test]
    fn slot_owner_rotates() {
        let set =
            ValidatorSet::new(vec!["v0".to_string(), "v1".to_string(), "v2".to_string()]).unwrap();
        assert_eq!(set.validator_for_slot(0), "v0");
        assert_eq!(set.validator_for_slot(1), "v1");
        assert_eq!(set.validator_for_slot(2), "v2");
        assert_eq!(set.validator_for_slot(3), "v0");
        assert_eq!(set.validator_for_slot(100), "v1");
    }
}
