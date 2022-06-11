pub enum IncomingQueueMessage {
    Halt,
    ChangeSleepIterCount(u64),
    ChangeSleepDurationMs(u64),
    ChangeMaxIter(u64),
}

pub enum OutgoingQueueMessage {
    Started,
    Iter(u64),
}
