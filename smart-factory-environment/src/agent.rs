use crate::event::{Event, EventArg};
use std::collections::HashMap;
use uuid::Uuid;

pub type NewEventsVec = Vec<(Event, u64)>;

pub trait Agent {
    fn handle(&mut self, time: u64, args: EventArg) -> NewEventsVec;

    fn solo_map(agent: &mut Self) -> HashMap<Uuid, &mut dyn Agent>
    where
        Self: Sized,
    {
        let mut map: HashMap<Uuid, &mut dyn Agent> = HashMap::new();
        map.insert(Uuid::new_v4(), agent);
        map
    }
}

pub trait AgentToMapExt<TAgent>
where
    TAgent: Agent,
{
    fn map_mut(&mut self) -> HashMap<Uuid, &mut dyn Agent>;
}

impl<TAgent> AgentToMapExt<TAgent> for Vec<TAgent>
where
    TAgent: Agent,
{
    fn map_mut(&mut self) -> HashMap<Uuid, &mut dyn Agent> {
        self.iter_mut()
            .map(|x| (Uuid::new_v4(), x as &mut dyn Agent))
            .collect()
    }
}
