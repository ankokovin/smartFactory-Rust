use crate::event_queue::EventEngineError;
use std::future::Future;
use std::pin::Pin;

pub const DEFAULT_ITER_COUNT_SLEEP: u64 = 5000;
pub const DEFAULT_SLEEP_DURATION_MS: u64 = 100;
pub const DEFAULT_MAX_ITER: u64 = u64::MAX;

pub trait EnvironmentSettings {
    fn get_iter_count(&self) -> u64 {
        DEFAULT_ITER_COUNT_SLEEP
    }
    fn get_sleep_ms(&self) -> u64 {
        DEFAULT_SLEEP_DURATION_MS
    }
    fn get_max_iter(&self) -> u64 {
        DEFAULT_MAX_ITER
    }
}

pub trait AgentEnvironment
{
    type LogFunction: FnMut(&str) + std::marker::Send;
    //FIXME:    I'm 99% sure that it is possible to express sleep function with only one type
    //          However when I half-heartedly tried to eliminate type SleepFuture compiler got angry
    //          Will probably fix later
    type SleepFunction: Fn(std::time::Duration) -> Self::SleepFuture;
    type SleepFuture: Future<Output = ()>;
    type TEnvironmentSettings: EnvironmentSettings;

    fn new(log: Self::LogFunction, sleep: Self::SleepFunction) -> Self;

    fn run(
        &mut self,
        settings: Self::TEnvironmentSettings,
    ) -> Pin<Box<dyn Future<Output = Result<(), EventEngineError>> + '_>>;

    fn halt(&mut self);

    fn change_sleep_time(&mut self, time_ms: u64);

    fn change_sleep_iter_count(&mut self, count: u64);

    fn change_max_iter_count(&mut self, count: u64);
}
