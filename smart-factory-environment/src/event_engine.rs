use crate::agent::Agent;
use crate::event::Event;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use uuid::Uuid;

pub struct EventEngine<'agent> {
    agents: HashMap<Uuid, &'agent mut dyn Agent>,
    queue: PriorityQueue<Event, u64>,
}

#[derive(Debug, PartialEq, Eq)]
enum EventEngineError {
    EventHasNoAgent,
}

#[allow(dead_code)]
impl<'agent> EventEngine<'agent> {
    fn new(agents: HashMap<Uuid, &'agent mut dyn Agent>) -> EventEngine<'agent> {
        EventEngine {
            agents,
            queue: priority_queue::PriorityQueue::new(),
        }
    }

    fn start(&mut self, init_state: Vec<(Event, u64)>) -> Result<(), EventEngineError> {
        self.queue.extend(init_state);
        loop {
            let item = self.queue.pop();
            if item.is_none() {
                return Ok(());
            }
            let (event, time) = item.unwrap();
            let agent = self.agents.get_mut(&event.agent);
            if agent.is_none() {
                return Err(EventEngineError::EventHasNoAgent);
            }
            let new_events = agent.unwrap().handle(time, event.args);
            self.queue.extend(new_events);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::agent::{Agent, AgentToMapExt, NewEventsVec};
    use crate::event::{EventArg, EventArgs};
    use crate::event_engine::{Event, EventEngine, EventEngineError};
    use priority_queue::PriorityQueue;
    use std::any::Any;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    pub fn it_errors_when_init_event_does_not_point_to_agent() {
        let events: Vec<(Event, u64)> = vec![(Event::new(Default::default()), 1)];
        let queue = PriorityQueue::new();
        let mut event_engine = EventEngine {
            queue,
            agents: HashMap::default(),
        };
        let result = event_engine.start(events);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EventEngineError::EventHasNoAgent);
    }

    #[test]
    pub fn it_calls_event_handler() {
        #[derive(Copy, Clone)]
        pub struct TestAgent {
            handler_was_called: bool,
        }

        impl Agent for TestAgent {
            fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
                self.handler_was_called = true;
                vec![]
            }
        }

        let agent_id = Uuid::new_v4();
        let mut agent = TestAgent {
            handler_was_called: false,
        };
        let mut agents = HashMap::new();
        agents.insert(agent_id, &mut agent as &mut dyn Agent);

        let queue = PriorityQueue::new();
        let mut event_engine = EventEngine { queue, agents };
        let result = event_engine.start(vec![(Event::new(agent_id), 0)]);

        assert!(result.is_ok());
        assert!(agent.handler_was_called);
    }

    #[test]
    pub fn test_new_several_agent_structs() {
        #[derive(Copy, Clone)]
        pub struct TestAgent {
            handler_was_called: bool,
        }

        impl Agent for TestAgent {
            fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
                self.handler_was_called = true;
                vec![]
            }
        }

        #[derive(Copy, Clone)]
        pub struct TestAgent2 {}

        impl Agent for TestAgent2 {
            fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
                vec![]
            }
        }

        let n = 5;
        let mut agents = vec![
            TestAgent {
                handler_was_called: false,
            };
            n
        ];

        let mut agents_map = agents.map_mut();
        let mut agents = vec![TestAgent2 {}; n];
        agents_map.extend(agents.map_mut());
        let engine = EventEngine::new(agents_map);
        assert_eq!(engine.agents.len(), 2 * n);
    }

    #[test]
    pub fn test_agent_event_generation() {
        struct CallerAgent {
            agent_to_call: Uuid,
        }

        impl Agent for CallerAgent {
            fn handle(&mut self, time: u64, _args: EventArg) -> NewEventsVec {
                vec![(Event::new(self.agent_to_call), time + 1)]
            }
        }

        struct CalleeAgent {
            was_called: bool,
        }

        impl Agent for CalleeAgent {
            fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
                self.was_called = true;
                vec![]
            }
        }

        let mut callee_agent = CalleeAgent { was_called: false };

        let callee_uuid = Uuid::new_v4();

        let mut caller_agent = CallerAgent {
            agent_to_call: callee_uuid,
        };

        let caller_uuid = Uuid::new_v4();

        let mut agents: HashMap<Uuid, &mut dyn Agent> = HashMap::new();

        agents.insert(caller_uuid, &mut caller_agent);
        agents.insert(callee_uuid, &mut callee_agent);

        let mut engine = EventEngine::new(agents);

        let init_state = vec![(Event::new(caller_uuid), 0)];

        let result = engine.start(init_state);
        assert!(result.is_ok());
        assert!(callee_agent.was_called);
    }

    #[test]
    pub fn test_event_args() {
        pub struct TestEventArg {
            x: u64,
        }

        impl EventArgs for TestEventArg {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct TestAgent {
            x: u64,
        }

        impl Agent for TestAgent {
            fn handle(&mut self, _time: u64, args: crate::event::EventArg) -> NewEventsVec {
                match args.unwrap().as_any().downcast_ref::<TestEventArg>() {
                    Some(arg) => {
                        self.x = arg.x;
                        vec![]
                    }
                    None => {
                        assert!(false, "Expected any to be EventArg");
                        vec![]
                    }
                }
            }
        }

        let mut agent = TestAgent { x: 0 };

        let mut engine = EventEngine::new(Agent::solo_map(&mut agent));

        let event = Event::new_with_args(
            *engine.agents.iter().next().unwrap().0,
            Box::new(TestEventArg { x: 42 }),
        );

        let result = engine.start(vec![(event, 0)]);
        assert!(result.is_ok());
        assert_eq!(agent.x, 42);
    }
}
