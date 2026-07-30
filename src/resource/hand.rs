use std::fmt;
use crate::Cost;
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
}
