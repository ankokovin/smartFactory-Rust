pub enum Message {
    Halt,
    ChangeSleepIterCount(u64),
    ChangeSleepDurationMs(u64),
}
