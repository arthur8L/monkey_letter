use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        // check empty str
        if name.trim().is_empty() {
            return Err(format!("{} is empty - not a valid subscriber name", name));
        }
        // check long name
        if name.graphemes(true).count() > 256 {
            return Err(format!(
                "{} is too long limit it to under 256 characters - not a valid subscriber name",
                name
            ));
        }
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        if name.chars().any(|c| forbidden_chars.contains(&c)) {
            return Err(format!(
                "{} contains invalid character(s) - not a valid subscriber name",
                name
            ));
        }
        Ok(Self(name))
    }
    pub fn inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "e".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_longer_name_over_256_grapheme_is_rejected() {
        let name = "e".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_names_rejected() {
        assert_err!(SubscriberName::parse(" ".to_string()));
    }

    #[test]
    fn empty_string_is_rejected() {
        assert_err!(SubscriberName::parse("".to_string()));
    }

    #[test]
    fn names_containing_an_invalid_char_is_rejected() {
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        for char in forbidden_chars {
            assert_err!(SubscriberName::parse(char.to_string()));
        }
    }

    #[test]
    fn valid_name_returns_success() {
        assert_ok!(SubscriberName::parse(String::from("Monkey Man")));
    }
}
