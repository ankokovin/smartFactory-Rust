use crate::event::{Event, EventArg};
use uuid::Uuid;

pub type NewEventsVec = Vec<(Event, u64)>;

pub trait Agent {
    fn handle(&mut self, time: u64, args: EventArg) -> NewEventsVec;

    fn get_id(&self) -> Uuid;

    fn solo_vec(agent: &mut Self) -> Vec<&mut dyn Agent>
    where
        Self: Sized,
    {
        vec![agent as &mut dyn Agent]
    }
}

pub trait AgentToMapExt<TAgent>
where
    TAgent: Agent,
{
    fn vec_mut(&mut self) -> Vec<&mut dyn Agent>;
}

impl<TAgent> AgentToMapExt<TAgent> for Vec<TAgent>
where
    TAgent: Agent,
{
    fn vec_mut(&mut self) -> Vec<&mut dyn Agent> {
        self.iter_mut().map(|x| x as &mut dyn Agent).collect()
    }
}
