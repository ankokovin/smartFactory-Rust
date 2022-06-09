use crate::agent::{Agent, AgentToMapExt};
use crate::environment::{AgentEnvironment, EnvironmentSettings};
use crate::event::{Event, EventArg};
use crate::event_queue::EventEngineError;
use crate::message::Message;
use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::time::Duration;
use uuid::Uuid;

pub struct EmptyEnvironmentSettings {
    pub agent_count: usize,
}

impl EnvironmentSettings for EmptyEnvironmentSettings {}

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
    sender: Option<Sender<Message>>,
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
            agents: vec![],
        }
    }

    fn run(
        &mut self,
        settings: &EmptyEnvironmentSettings,
    ) -> Pin<Box<dyn Future<Output = Result<(), EventEngineError>> + '_>> {
        self.agents = vec![InfiniteLoopAgent::new(); settings.agent_count];
        (self.log)("Starting");
        let (sender, receiver) = mpsc::channel::<Message>();
        self.sender = Some(sender.clone());
        let event_vec = self
            .agents
            .iter()
            .map(|agent| (Event::new(agent.id), 0))
            .collect();
        let agent_vec = self.agents.vec_mut();
        (self.sleep)(Duration::from_millis(100));
        return Box::pin(crate::event_queue::process_event_queue(
            agent_vec,
            event_vec,
            receiver,
            &mut self.log,
            &mut self.sleep,
        ));
    }

    fn halt(&mut self) {
        if self.sender.is_some() {
            (self.log)("Halting");
            //FIXME: handle error somehow?
            let _send_result = self.sender.as_ref().unwrap().send(Message::Halt);
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
            environment.run(&EmptyEnvironmentSettings { agent_count: 42 }),
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
        let run = environment.run(&EmptyEnvironmentSettings { agent_count: 1 });

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
