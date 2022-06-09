use crate::event_queue::EventEngineError;
use std::future::Future;
use std::pin::Pin;

pub trait EnvironmentSettings {}

pub trait AgentEnvironment<LogFunction, SleepFunction, SleepFut, TEnvironmentSettings>
where
    LogFunction: FnMut(&str) + std::marker::Send,
    SleepFunction: Fn(std::time::Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
    TEnvironmentSettings: EnvironmentSettings,
{
    fn new(log: LogFunction, sleep: SleepFunction) -> Self;

    fn run(
        &mut self,
        settings: &TEnvironmentSettings,
    ) -> Pin<Box<dyn Future<Output = Result<(), EventEngineError>> + '_>>;

    fn halt(&mut self);
}
