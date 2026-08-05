use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NumberToken(u8);

// board/tile/token.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNumberToken(pub u8);

impl fmt::Display for InvalidNumberToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} n'est pas un jeton valide (2-12 sauf 7)", self.0)
    }
}
impl std::error::Error for InvalidNumberToken {}

impl NumberToken {
    pub(crate) fn new(n: u8) -> Result<Self, InvalidNumberToken> {
        if matches!(n, 2..=6 | 8..=12) {
            Ok(Self(n))
        } else {
            Err(InvalidNumberToken(n))
        }
    }

    pub(crate) fn value(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_numbers(){
        assert!(NumberToken::new(2).is_ok());
        assert!(NumberToken::new(6).is_ok());
        assert!(NumberToken::new(8).is_ok());
        assert!(NumberToken::new(12).is_ok());
    }

    #[test]
    fn checks_value() {
        let t = NumberToken::new(2).unwrap();
        assert_eq!(t.value(), 2);
    }

    #[test]
    fn rejects_invalid_numbers() {
        assert!(NumberToken::new(1).is_err());
        assert!(NumberToken::new(7).is_err());
        assert!(NumberToken::new(13).is_err());
    }
}