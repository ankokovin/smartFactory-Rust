use crate::agent::{Agent, AgentToMapExt};
use crate::environment::{AgentEnvironment, EnvironmentSettings};
use crate::event::{Event, EventArg};
use crate::event_queue::EventEngineError;
use crate::message::{IncomingQueueMessage, OutgoingQueueMessage};
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use uuid::Uuid;

pub struct EmptyEnvironmentSettings {
    agent_count: usize,
    sleep_ms: u64,
    iter_count: u64,
}

impl EmptyEnvironmentSettings {
    pub fn new(agent_count: usize, sleep_ms: u64, iter_count: u64) -> EmptyEnvironmentSettings {
        EmptyEnvironmentSettings {
            agent_count,
            sleep_ms,
            iter_count,
        }
    }
}

impl EnvironmentSettings for EmptyEnvironmentSettings {
    fn get_iter_count(&self) -> u64 {
        self.iter_count
    }

    fn get_sleep_ms(&self) -> u64 {
        self.sleep_ms
    }
}

#[derive(Clone, Copy)]
pub struct InfiniteLoopAgent {
    id: Uuid,
    pub was_called: bool,
}

impl Agent for InfiniteLoopAgent {
    fn handle(&mut self, time: u64, _args: EventArg) -> crate::agent::NewEventsVec {
        self.was_called = true;
        vec![(Event::new(self.id), time + 1)]
    }

    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl InfiniteLoopAgent {
    fn new() -> InfiniteLoopAgent {
        InfiniteLoopAgent {
            id: Uuid::new_v4(),
            was_called: false,
        }
    }
}

pub struct InfiniteEmptyEnvironment<LogFunction, SleepFunction, SleepFut>
where
    LogFunction: FnMut(&str),
    SleepFunction: Fn(std::time::Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    log: LogFunction,
    sleep: SleepFunction,
    sender: Option<Sender<IncomingQueueMessage>>,
    pub receiver: Option<Receiver<OutgoingQueueMessage>>,
    agents: Vec<InfiniteLoopAgent>,
}

impl<LogFunction, SleepFunction, SleepFut>
    AgentEnvironment<LogFunction, SleepFunction, SleepFut, EmptyEnvironmentSettings>
    for InfiniteEmptyEnvironment<LogFunction, SleepFunction, SleepFut>
where
    LogFunction: FnMut(&str) + std::marker::Send,
    SleepFunction: Fn(std::time::Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    fn new(mut log: LogFunction, sleep: SleepFunction) -> Self {
        log("Creating new environment");
        Self {
            log,
            sleep,
            sender: None,
            receiver: None,
            agents: vec![],
        }
    }

    fn run(
        &mut self,
        settings: EmptyEnvironmentSettings,
    ) -> Pin<Box<dyn Future<Output = Result<(), EventEngineError>> + '_>> {
        self.agents = vec![InfiniteLoopAgent::new(); settings.agent_count];
        (self.log)("Starting");
        let (in_sender, in_receiver) = mpsc::channel();
        self.sender = Some(in_sender);
        let (out_sender, out_receiver) = mpsc::channel();
        self.receiver = Some(out_receiver);
        let event_vec = self
            .agents
            .iter()
            .map(|agent| (Event::new(agent.id), 0))
            .collect();
        let agent_vec = self.agents.mut_agent_vector();
        (self.sleep)(Duration::from_millis(100));
        return Box::pin(crate::event_queue::process_event_queue(
            agent_vec,
            event_vec,
            in_receiver,
            &mut self.log,
            &mut self.sleep,
            settings,
            out_sender,
        ));
    }

    fn halt(&mut self) {
        if self.sender.is_some() {
            (self.log)("Halting");
            //FIXME: handle error somehow?
            let _send_result = self
                .sender
                .as_ref()
                .unwrap()
                .send(IncomingQueueMessage::Halt);
            self.sender = None
        }
    }

    fn change_sleep_time(&mut self, time_ms: u64) {
        if self.sender.is_some() {
            (self.log)("Changing sleep time");
            //FIXME: handle error somehow?
            let _send_result = self
                .sender
                .as_ref()
                .unwrap()
                .send(IncomingQueueMessage::ChangeSleepDurationMs(time_ms));
            self.sender = None
        }
    }

    fn change_sleep_iter_count(&mut self, count: u64) {
        if self.sender.is_some() {
            (self.log)("Changing sleep iter count");
            //FIXME: handle error somehow?
            let _send_result = self
                .sender
                .as_ref()
                .unwrap()
                .send(IncomingQueueMessage::ChangeSleepIterCount(count));
            self.sender = None
        }
    }

    fn change_max_iter_count(&mut self, count: u64) {
        if self.sender.is_some() {
            (self.log)("Changing max iter count");
            //FIXME: handle error somehow?
            let _send_result = self
                .sender
                .as_ref()
                .unwrap()
                .send(IncomingQueueMessage::ChangeMaxIter(count));
            self.sender = None
        }
    }
}

impl<LogFunction, SleepFunction, SleepFut>
    InfiniteEmptyEnvironment<LogFunction, SleepFunction, SleepFut>
where
    LogFunction: FnMut(&str),
    SleepFunction: Fn(std::time::Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    pub fn get_agents(&self) -> Vec<InfiniteLoopAgent> {
        self.agents.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::AgentEnvironment;
    use futures::pin_mut;
    use std::time::Duration;

    const ITER_COUNT_SLEEP: u64 = 5000;
    const SLEEP_DURATION_MS: u64 = 100;

    #[tokio::test]
    pub async fn it_runs_a_lot_without_halting() {
        let log_function = |message: &str| {
            println!("{}", message);
        };
        let sleep_function = |duration| tokio::time::sleep(duration);
        let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep_function);
        //assert_eq!(log_message.clone(), "Creating new environment");
        let t = tokio::time::timeout(
            Duration::from_secs(1),
            //assert_eq!(log_message.clone(), "Starting");
            environment.run(EmptyEnvironmentSettings {
                agent_count: 42,
                sleep_ms: SLEEP_DURATION_MS,
                iter_count: ITER_COUNT_SLEEP,
            }),
        );
        let result = t.await;
        assert!(result.is_err())
    }

    #[tokio::test]
    pub async fn when_halting_then_call_log() {
        let mut log_message = String::new();
        let log_function = |message: &str| {
            println!("{}", message);
            log_message = message.to_string();
        };
        let sleep_function = |duration| tokio::time::sleep(duration);
        let mut environment = InfiniteEmptyEnvironment::new(log_function, sleep_function);
        let run = environment.run(EmptyEnvironmentSettings {
            agent_count: 1,
            sleep_ms: SLEEP_DURATION_MS,
            iter_count: ITER_COUNT_SLEEP,
        });

        let wait = tokio::time::sleep(Duration::from_secs(1));
        pin_mut!(wait);
        let sel = futures::future::select(run, wait);
        sel.await;
        environment.halt();
        environment.get_agents().iter().for_each(|agent| {
            assert!(agent.was_called);
        });
        assert_eq!(log_message, "Halting");
    }
}
