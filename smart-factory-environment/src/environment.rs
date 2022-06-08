pub trait EnvironmentSettings {}

pub trait AgentEnvironment<LogFunction, TEnvironmentSettings>
where
    LogFunction: FnMut(&str),
    TEnvironmentSettings: EnvironmentSettings,
{
    fn new(settings: &TEnvironmentSettings, log: LogFunction) -> Self;

    fn run(&mut self);

    fn halt(&mut self);
}
