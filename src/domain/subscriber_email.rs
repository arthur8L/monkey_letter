use validator::ValidateEmail;
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<SubscriberEmail, String> {
        if !ValidateEmail::validate_email(&email) {
            return Err(format!("{} is not a valid subscriber email.", email));
        }
        Ok(Self(email))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // https://github.com/LukeMathWalker/zero-to-production/issues/34
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            Self(SafeEmail().fake_with_rng(&mut rng))
        }
    }
    #[test]
    fn empty_string_is_rejected() {
        assert_err!(SubscriberEmail::parse("".to_string()));
    }

    #[test]
    fn email_missing_at_symbol_throws() {
        assert_err!(SubscriberEmail::parse("test-test.com".to_string()));
    }

    #[test]
    fn email_missing_subject_throws() {
        assert_err!(SubscriberEmail::parse("@test.com".to_string()));
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_passed(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
