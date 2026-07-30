use std::fmt;
use crate::{Cost, Resource};
use crate::resource::counts::ResourceCounts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotEnoughResources;

impl fmt::Display for NotEnoughResources {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ressources insuffisantes")
    }
}
impl std::error::Error for NotEnoughResources {}

#[derive(Default)]
pub struct Hand(ResourceCounts);

impl Hand {
    pub fn pay(&mut self, cost: &Cost) -> Result<(), NotEnoughResources> {
        if !self.0.try_subtract(&cost.resources()) {
            return Err(NotEnoughResources);
        }
        Ok(())
    }

    pub fn resources(&self) -> ResourceCounts {
        self.0
    }

    pub fn count(&self) -> u8 {
        self.0.count()
    }

    pub fn add(&mut self, resource : Resource, amount : u8){
        self.0.add(resource, amount);
    }


}

#[cfg(test)]
mod tests {
    use crate::Resource;
    use crate::Gain;
    use crate::player::PlayerId;
    use super::*;

    #[test]
    fn test_hand_pay_with_not_enough_resources() -> () {
        let mut hand = Hand::default();
        assert_eq!(hand.pay(&Cost::ROAD), Err(NotEnoughResources));
        assert_eq!(hand.resources(), ResourceCounts::default());
    }

    #[test]
    fn test_hand_pay_with_enough_resources() -> () {
        let mut hand = Hand::default();
        hand.add(Resource::Brick, 1);
        hand.add(Resource::Wood, 1);
        assert_eq!(hand.pay(&Cost::ROAD), Ok(()));
        assert_eq!(hand.resources(), ResourceCounts::default());
    }

    #[test]
    fn test_hand_add() -> () {
        let mut hand = Hand::default();
        hand.add(Resource::Brick, 1);
        assert_eq!(hand.resources(), ResourceCounts::new([0, 0, 1, 0, 0]));
    }
}
