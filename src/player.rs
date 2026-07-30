use crate::{Hand};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlayerColor { Red, Blue, White, Orange }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(u8);
impl PlayerId {
    pub fn new(id: u8) -> Self {
        Self(id)
    }
}


pub struct Player {
    color : PlayerColor,
    hand : Hand
}

impl Player {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_id() {
        assert_eq!(PlayerId::new(0), PlayerId(0));
    }


    #[test]
    fn test_player_color(){
        let player = Player { color: PlayerColor::Red, hand: Hand::default() };
        assert_eq!(player.color, PlayerColor::Red);
    }
}