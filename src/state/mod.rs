mod history;
mod jobs;

pub use history::HistoryState;
pub use jobs::{Job, JobState, JobStatus};

pub struct ShellState {
    pub history: HistoryState,
    pub jobs: JobState,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            history: HistoryState::new(),
            jobs: JobState::new(),
        }
    }
}

impl ShellState {
    pub fn new() -> Self {
        Self::default()
    }
}
