use crate::{Hand, ResourceError, Resource, ResourceCounts};

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

    pub(crate) fn value(self) -> usize {
        self.0
    }
}

pub struct Player {
    color: PlayerColor,
    hand: Hand,
    score: u8,
}

impl Player {
    pub fn new(color: PlayerColor) -> Self {
        Self {
            color,
            hand: Hand::default(),
            score: 0,
        }
    }

    pub fn color(&self) -> PlayerColor {
        self.color
    }

    pub(crate) fn hand(&self) -> &Hand {
        &self.hand
    }

    pub(crate) fn receive(&mut self, resources : ResourceCounts) {
        
        self.hand.add(resources);
    }

    pub(crate) fn pay(&mut self, cost : &crate::Cost) -> Result<(), ResourceError> {
        self.hand.pay(cost)
    }

    pub(crate) fn score(&self) -> u8 {
        self.score
    }

    pub(crate) fn add_score(&mut self, amount : u8){
        self.score += amount;
    }

    pub(crate) fn remove_score(&mut self, amount : u8){
        self.score -= amount;
    }
}
