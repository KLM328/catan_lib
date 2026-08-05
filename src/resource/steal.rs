use crate::PlayerId;

pub struct Steal {victim : PlayerId, resource : Option<u8>}

impl Steal {
    pub fn new(victim : PlayerId, resource : Option<u8>) -> Steal {
        Steal {victim, resource}
    }

    pub fn victim(&self) -> PlayerId {
        self.victim
    }

    pub fn resource(&self) -> Option<u8> {
        self.resource
    }
}