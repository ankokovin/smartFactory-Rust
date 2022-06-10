pub enum Message {
    Halt,
    ChangeSleepIterCount(u64),
    ChangeSleepDurationMs(u64),
    ChangeMaxIter(u64),
}
