use crate::agent::Agent;
use crate::environment::EnvironmentSettings;
use crate::event::Event;
use crate::message::{IncomingQueueMessage, OutgoingQueueMessage};
use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::future::Future;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::time::Duration;
use uuid::Uuid;

impl From<SendError<OutgoingQueueMessage>> for EventEngineError {
    fn from(err: SendError<OutgoingQueueMessage>) -> Self {
        EventEngineError::CouldNotCommunicate(err)
    }
}

#[derive(Debug)]
pub enum EventEngineError {
    EventHasNoAgent,
    CouldNotCommunicate(SendError<OutgoingQueueMessage>)
}

pub async fn process_event_queue<LogFunction, SleepFunction, SleepFut, Settings>(
    mut agents: Vec<&mut dyn Agent>,
    init_state: Vec<(Event, u64)>,
    receiver: Receiver<IncomingQueueMessage>,
    log: &mut LogFunction,
    sleep: &mut SleepFunction,
    settings: Settings,
    sender: Sender<OutgoingQueueMessage>,
) -> Result<(), EventEngineError>
where
    LogFunction: FnMut(&str),
    SleepFunction: Fn(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
    Settings: EnvironmentSettings,
{
    let mut queue = PriorityQueue::new();
    queue.extend(init_state);
    let mut agents: HashMap<Uuid, &mut &mut dyn Agent> = agents
        .iter_mut()
        .map(|agent| (agent.get_id(), agent))
        .collect();
    let mut i = 0;
    let mut sleep_duration = Duration::from_millis(settings.get_sleep_ms());
    let mut max_iter_count = settings.get_max_iter();
    let mut iter_count_sleep = settings.get_iter_count();


    sender.send(OutgoingQueueMessage::Started)?;
    loop {
        if let Ok(message) = receiver.try_recv() {
            match message {
                IncomingQueueMessage::Halt => return Ok(()),
                IncomingQueueMessage::ChangeSleepIterCount(count) => iter_count_sleep = count,
                IncomingQueueMessage::ChangeSleepDurationMs(sleep_ms) => {
                    sleep_duration = Duration::from_millis(sleep_ms)
                }
                IncomingQueueMessage::ChangeMaxIter(count) => max_iter_count = count,
            }
        }

        if i >= max_iter_count {
            return Ok(());
        }

        let item = queue.pop();
        if item.is_none() {
            return Ok(());
        }
        let (event, time) = item.unwrap();
        let agent = agents.get_mut(&event.agent);
        if agent.is_none() {
            return Err(EventEngineError::EventHasNoAgent);
        }
        let new_events = agent.unwrap().handle(time, event.args);
        queue.extend(new_events);

        i += 1;

        if i % iter_count_sleep == 0 {
            sender.send(OutgoingQueueMessage::Iter(i))?;
            (log)("Entered sleep");
            (sleep)(sleep_duration).await;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::agent::{Agent, AgentToMapExt, NewEventsVec};
    use crate::event::{Event, EventArg, EventArgs};
    use crate::event_queue::{process_event_queue, EventEngineError};

    use crate::environment::{EnvironmentSettings, DEFAULT_ITER_COUNT_SLEEP};
    use crate::message::{IncomingQueueMessage, OutgoingQueueMessage};
    use std::any::Any;
    use std::sync::mpsc;
    use std::time::Duration;
    use uuid::Uuid;

    struct TestSettings {}
    impl EnvironmentSettings for TestSettings {}

    #[derive(Copy, Clone)]
    pub struct TestAgentWasCalled {
        id: Uuid,
        handler_was_called: bool,
    }

    impl Agent for TestAgentWasCalled {
        fn handle(&mut self, _time: u64, _args: EventArg) -> NewEventsVec {
            self.handler_was_called = true;
            vec![]
        }

        fn get_id(&self) -> Uuid {
            self.id
        }
    }

    #[tokio::test]
    pub async fn it_errors_when_init_event_does_not_point_to_agent() {
        let events: Vec<(Event, u64)> = vec![(Event::new(Default::default()), 1)];
        let agents = vec![];
        let (_send, recv) = mpsc::channel();
        let (send, _recv) = mpsc::channel();
        let result = process_event_queue(
            agents,
            events,
            recv,
            &mut |_| {},
            &mut |_| async {},
            TestSettings {},
            send,
        )
        .await;
        assert!(matches!(result, Err(EventEngineError::EventHasNoAgent)));
    }

    #[tokio::test]
    pub async fn it_calls_event_handler() {
        let agent_id = Uuid::new_v4();
        let mut agent = TestAgentWasCalled {
            id: agent_id,
            handler_was_called: false,
        };
        let agents = vec![&mut agent as &mut dyn Agent];

        let (_send, recv) = mpsc::channel();
        let (out_send, out_recv) = mpsc::channel();

        let result = process_event_queue(
            agents,
            vec![(Event::new(agent_id), 0)],
            recv,
            &mut |_| {},
            &mut |_| async {},
            TestSettings {},
            out_send,
        )
        .await;

        assert!(result.is_ok());
        assert!(agent.handler_was_called);
        let result = out_recv.try_recv();
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), OutgoingQueueMessage::Started));
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

        let callee_uuid = Uuid::new_v4();
        let mut callee_agent = TestAgentWasCalled {
            id: callee_uuid,
            handler_was_called: false,
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
        let (send, _recv) = mpsc::channel();

        let result = process_event_queue(
            agents,
            init_state,
            recv,
            &mut |_| {},
            &mut |_| async {},
            TestSettings {},
            send,
        )
        .await;
        assert!(result.is_ok());
        assert!(callee_agent.handler_was_called);
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
        let (send, _recv) = mpsc::channel();

        let result = process_event_queue(
            agents,
            vec![(event, 0)],
            recv,
            &mut |_| {},
            &mut |_| async {},
            TestSettings {},
            send,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(agent.x, 42);
    }

    #[tokio::test]
    pub async fn test_event_args_diff_types() {
        pub struct EventInc {
            x: i64,
        }

        impl EventArgs for EventInc {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct EventDec {
            x: i64,
        }

        impl EventArgs for EventDec {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        pub struct TestAgent {
            x: i64,
            id: Uuid,
        }

        impl Agent for TestAgent {
            fn handle(&mut self, _time: u64, args: crate::event::EventArg) -> NewEventsVec {
                assert!(args.is_some());
                let args = args.unwrap();
                let args = args.as_any();
                if let Some(arg) = args.downcast_ref::<EventInc>() {
                    self.x += arg.x;
                    vec![]
                } else if let Some(arg) = args.downcast_ref::<EventDec>() {
                    self.x -= arg.x;
                    vec![]
                } else {
                    panic!()
                }
            }

            fn get_id(&self) -> Uuid {
                self.id
            }
        }

        let agent_id_inc = Uuid::new_v4();
        let agent_id_dec = Uuid::new_v4();

        let mut agents = vec![
            TestAgent {
                x: 0,
                id: agent_id_inc,
            },
            TestAgent {
                x: 0,
                id: agent_id_dec,
            },
        ];

        let events = vec![
            (
                Event::new_with_args(agent_id_inc, Box::new(EventInc { x: 42 })),
                0,
            ),
            (
                Event::new_with_args(agent_id_dec, Box::new(EventDec { x: 42 })),
                0,
            ),
        ];

        let (_send, recv) = mpsc::channel();
        let (send, _recv) = mpsc::channel();

        let result = process_event_queue(
            agents.mut_agent_vector(),
            events,
            recv,
            &mut |_| {},
            &mut |_| async {},
            TestSettings {},
            send,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(agents.get(0).unwrap().x, 42);
        assert_eq!(agents.get(1).unwrap().x, -42);
    }

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

    #[tokio::test]
    pub async fn test_kill_from_halt_message() {
        let id = Uuid::new_v4();

        let mut agent = InfiniteLoopAgent { id };

        let agents = Agent::solo_vec(&mut agent);

        let event = Event::new(id);

        let (send, recv) = mpsc::channel();
        let (outsend, _recv) = mpsc::channel();
        let send_result = send.send(IncomingQueueMessage::Halt);
        assert!(send_result.is_ok());
        let result = process_event_queue(
            agents,
            vec![(event, 0)],
            recv,
            &mut |_| {},
            &mut |duration| tokio::time::sleep(duration),
            TestSettings {},
            outsend,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    pub async fn test_not_halt_does_not_kill() {
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
                let id = Uuid::new_v4();

                let mut agent = InfiniteLoopAgent { id };

                let agents = Agent::solo_vec(&mut agent);

                let event = Event::new(id);

                let (send, recv) = mpsc::channel();
                let (outsend, _recv) = mpsc::channel();

                let result = process_event_queue(
                    agents,
                    vec![(event, 0)],
                    recv,
                    &mut self.log,
                    &mut self.sleep,
                    TestSettings {},
                    outsend,
                );

                let send_result = send.send(IncomingQueueMessage::ChangeSleepDurationMs(50000));
                assert!(send_result.is_ok());

                let t = tokio::time::timeout(Duration::from_secs(1), result);
                let result = t.await;
                assert!(result.is_err());
            }
        }

        let mut t = Test {
            log: |_| {},
            sleep: |duration| tokio::time::sleep(duration),
        };
        t.run().await;
    }

    #[tokio::test]
    pub async fn it_respects_max_iter_count() {
        struct TestSettingsZeroMaxIter {}

        impl EnvironmentSettings for TestSettingsZeroMaxIter {
            fn get_max_iter(&self) -> u64 {
                0
            }
        }

        let agent_id = Uuid::new_v4();
        let mut agent = TestAgentWasCalled {
            id: agent_id,
            handler_was_called: false,
        };
        let agents = vec![&mut agent as &mut dyn Agent];

        let (_send, recv) = mpsc::channel();
        let (outsend, _recv) = mpsc::channel();

        let result = process_event_queue(
            agents,
            vec![(Event::new(agent_id), 0)],
            recv,
            &mut |_| {},
            &mut |_| async {},
            TestSettingsZeroMaxIter {},
            outsend,
        )
        .await;

        assert!(result.is_ok());
        assert!(!agent.handler_was_called);
    }

    #[tokio::test]
    pub async fn test_kill_from_max_iter_message() {
        let id = Uuid::new_v4();

        let mut agent = InfiniteLoopAgent { id };

        let agents = Agent::solo_vec(&mut agent);

        let event = Event::new(id);

        let (send, recv) = mpsc::channel();
        let (outsend, outrecv) = mpsc::channel();

        let send_result = send.send(IncomingQueueMessage::ChangeMaxIter(
            DEFAULT_ITER_COUNT_SLEEP * 2 - 1,
        ));
        assert!(send_result.is_ok());
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            process_event_queue(
                agents,
                vec![(event, 0)],
                recv,
                &mut |_| {},
                &mut |duration| tokio::time::sleep(duration),
                TestSettings {},
                outsend,
            ),
        )
        .await;
        assert!(result.is_ok());
        let result = outrecv.try_recv();
        assert!(result.is_ok());

        if let OutgoingQueueMessage::Iter(iter) = result.unwrap() {
            assert_eq!(iter, DEFAULT_ITER_COUNT_SLEEP);
        }
    }
}
