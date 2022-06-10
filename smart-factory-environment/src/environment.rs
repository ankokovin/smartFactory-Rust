use crate::event_queue::EventEngineError;
use std::future::Future;
use std::pin::Pin;

const ITER_COUNT_SLEEP: u64 = 5000;
const SLEEP_DURATION_MS: u64 = 100;

pub trait EnvironmentSettings {
    fn get_iter_count(&self) -> u64 {
        ITER_COUNT_SLEEP
    }
    fn get_sleep_ms(&self) -> u64 {
        SLEEP_DURATION_MS
    }
    fn get_max_iter(&self) -> u64 {
        u64::MAX
    }
}

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
        settings: TEnvironmentSettings,
    ) -> Pin<Box<dyn Future<Output = Result<(), EventEngineError>> + '_>>;

    fn halt(&mut self);

    fn change_sleep_time(&mut self, time_ms: u64);

    fn change_sleep_iter_count(&mut self, count: u64);

    fn change_max_iter_count(&mut self, count: u64);
}
