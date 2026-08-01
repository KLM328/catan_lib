use crate::{Hand, Resource};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PlayerColor {
    Red,
    Blue,
    White,
    Orange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(usize);
impl PlayerId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn id(self) -> usize {
        self.0
    }
}

pub struct Player {
    color: PlayerColor,
    hand: Hand,
}

impl Player {
    pub(crate) fn new(color: PlayerColor) -> Self {
        Self {
            color,
            hand: Hand::default(),
        }
    }

    pub(crate) fn color(&self) -> PlayerColor {
        self.color
    }

    pub(crate) fn hand(&self) -> &Hand {
        &self.hand
    }

    pub(crate) fn receive(&mut self, resource : Resource, amount : u8){
        self.hand.add(resource, amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_id() {
        assert_eq!(PlayerId::new(0), PlayerId(0));
    }

    #[test]
    fn test_player_color() {
        let player = Player {
            color: PlayerColor::Red,
            hand: Hand::default(),
        };
        assert_eq!(player.color, PlayerColor::Red);
    }
}
