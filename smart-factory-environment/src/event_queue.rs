use crate::agent::Agent;
use crate::event::Event;
use crate::message::Message;
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::future::Future;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum EventEngineError {
    EventHasNoAgent,
}

const ITER_COUNT_SLEEP: u64 = 5000;
const SLEEP_DURATION_MS: u64 = 100;

pub async fn process_event_queue<LogFunction, SleepFunction, SleepFut>(
    mut agents: Vec<&mut dyn Agent>,
    init_state: Vec<(Event, u64)>,
    receiver: Receiver<Message>,
    log: &mut LogFunction,
    sleep: &mut SleepFunction,
) -> Result<(), EventEngineError>
where
    LogFunction: FnMut(&str),
    SleepFunction: Fn(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    let mut queue = PriorityQueue::new();
    queue.extend(init_state);
    let mut agents: HashMap<Uuid, &mut &mut dyn Agent> = agents
        .iter_mut()
        .map(|agent| (agent.get_id(), agent))
        .collect();
    let mut i = 0;
    loop {
        let item = queue.pop();
        if item.is_none() {
            return Ok(());
        }
        let (event, time) = item.unwrap();
        let mut agent = agents.get_mut(&event.agent);
        let agent = agent.as_deref_mut();
        if agent.is_none() {
            return Err(EventEngineError::EventHasNoAgent);
        }
        let new_events = agent.unwrap().handle(time, event.args);
        queue.extend(new_events);

        if let Ok(_ans) = receiver.try_recv() {
            return Ok(());
        }

        i += 1;
        if i >= ITER_COUNT_SLEEP {
            (log)("Entered sleep");
            (sleep)(Duration::from_millis(SLEEP_DURATION_MS)).await;
            i = 0;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::agent::{Agent, NewEventsVec};
    use crate::event::{Event, EventArg, EventArgs};
    use crate::event_queue::{process_event_queue, EventEngineError};

    use crate::message::Message;
    use std::any::Any;
    use std::sync::mpsc;
    use uuid::Uuid;

    #[tokio::test]
    pub async fn it_errors_when_init_event_does_not_point_to_agent() {
        let events: Vec<(Event, u64)> = vec![(Event::new(Default::default()), 1)];
        let agents = vec![];
        let (_send, recv) = mpsc::channel();
        let result =
            process_event_queue(agents, events, recv, &mut |_| {}, &mut |_| async {}).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), EventEngineError::EventHasNoAgent);
    }

    #[tokio::test]
    pub async fn it_calls_event_handler() {
        #[derive(Copy, Clone)]
        pub struct TestAgent {
            id: Uuid,
            handler_was_called: bool,
        }

        impl Agent for TestAgent {
            fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
                self.handler_was_called = true;
                vec![]
            }

            fn get_id(&self) -> Uuid {
                self.id
            }
        }

        let agent_id = Uuid::new_v4();
        let mut agent = TestAgent {
            id: agent_id,
            handler_was_called: false,
        };
        let agents = vec![&mut agent as &mut dyn Agent];

        let (_send, recv) = mpsc::channel();
        let result = process_event_queue(
            agents,
            vec![(Event::new(agent_id), 0)],
            recv,
            &mut |_| {},
            &mut |_| async {},
        )
        .await;

        assert!(result.is_ok());
        assert!(agent.handler_was_called);
    }

    #[tokio::test]
    pub async fn test_agent_event_generation() {
        struct CallerAgent {
            agent_to_call: Uuid,
            id: Uuid,
        }

        impl Agent for CallerAgent {
            fn handle(&mut self, time: u64, _args: EventArg) -> NewEventsVec {
                vec![(Event::new(self.agent_to_call), time + 1)]
            }

            fn get_id(&self) -> Uuid {
                self.id
            }
        }

        struct CalleeAgent {
            id: Uuid,
            was_called: bool,
        }

        impl Agent for CalleeAgent {
            fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
                self.was_called = true;
                vec![]
            }

            fn get_id(&self) -> Uuid {
                self.id
            }
        }

        let callee_uuid = Uuid::new_v4();
        let mut callee_agent = CalleeAgent {
            id: callee_uuid,
            was_called: false,
        };

        let caller_uuid = Uuid::new_v4();
        let mut caller_agent = CallerAgent {
            id: caller_uuid,
            agent_to_call: callee_uuid,
        };

        let agents: Vec<&mut dyn Agent> = vec![
            &mut caller_agent as &mut dyn Agent,
            &mut callee_agent as &mut dyn Agent,
        ];

        let init_state = vec![(Event::new(caller_uuid), 0)];

        let (_send, recv) = mpsc::channel();
        let result =
            process_event_queue(agents, init_state, recv, &mut |_| {}, &mut |_| async {}).await;
        assert!(result.is_ok());
        assert!(callee_agent.was_called);
    }

    #[tokio::test]
    pub async fn test_event_args() {
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
            id: Uuid,
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

            fn get_id(&self) -> Uuid {
                self.id
            }
        }

        let mut agent = TestAgent {
            x: 0,
            id: Uuid::new_v4(),
        };

        let event = Event::new_with_args(agent.get_id(), Box::new(TestEventArg { x: 42 }));
        let agents = Agent::solo_vec(&mut agent);

        let (_send, recv) = mpsc::channel();
        let result = process_event_queue(
            agents,
            vec![(event, 0)],
            recv,
            &mut |_| {},
            &mut |_| async {},
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(agent.x, 42);
    }

    #[tokio::test]
    pub async fn test_kill_from_message() {
        pub struct Test<LogFunction, SleepFunction, SleepFut>
        where
            LogFunction: FnMut(&str),
            SleepFunction: Fn(std::time::Duration) -> SleepFut,
            SleepFut: std::future::Future<Output = ()>,
        {
            log: LogFunction,
            sleep: SleepFunction,
        }

        impl<LogFunction, SleepFunction, SleepFut> Test<LogFunction, SleepFunction, SleepFut>
        where
            LogFunction: FnMut(&str),
            SleepFunction: Fn(std::time::Duration) -> SleepFut,
            SleepFut: std::future::Future<Output = ()>,
        {
            async fn run(&mut self) {
                pub struct InfiniteLoopAgent {
                    id: Uuid,
                }

                impl Agent for InfiniteLoopAgent {
                    fn handle(&mut self, time: u64, _args: EventArg) -> NewEventsVec {
                        vec![(Event::new(self.id), time + 1)]
                    }

                    fn get_id(&self) -> Uuid {
                        self.id
                    }
                }

                let id = Uuid::new_v4();

                let mut agent = InfiniteLoopAgent { id };

                let agents = Agent::solo_vec(&mut agent);

                let event = Event::new(id);

                let (send, recv) = mpsc::channel();
                let result = process_event_queue(
                    agents,
                    vec![(event, 0)],
                    recv,
                    &mut self.log,
                    &mut self.sleep,
                );

                let send_result = send.send(Message::Halt);
                assert!(send_result.is_ok());

                let result = result.await;
                assert!(result.is_ok());
            }
        }

        let mut t = Test {
            log: |_| {},
            sleep: |_| async {},
        };
        t.run().await;
    }
}
