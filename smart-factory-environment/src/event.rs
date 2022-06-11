use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub type EventArg = Option<Box<dyn EventArgs>>;

pub struct Event {
    id: Uuid,
    pub agent: Uuid,
    pub args: EventArg,
}

impl Event {
    pub fn new(agent: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent,
            args: None,
        }
    }

    pub fn new_with_args(agent: Uuid, args: Box<dyn EventArgs>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent,
            args: Some(args),
        }
    }
}

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.id, self.agent).hash(state)
    }
}

impl PartialEq<Self> for Event {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Event {}

pub trait EventArgs {
    fn as_any(&self) -> &dyn std::any::Any;
}
