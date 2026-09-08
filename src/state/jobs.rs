use std::{io::Write, process::Child};
use strum::Display;

#[derive(Debug, Default, PartialEq, Eq, Display)]
pub enum JobStatus {
    #[default]
    #[strum(to_string = "running")]
    Running,
    #[strum(to_string = "done")]
    Done,
    #[strum(to_string = "failed")]
    Failed(i32),
}

#[derive(Debug)]
pub struct Job {
    status: JobStatus,
    pid: u32,
    id: u32,
    child: Child,
    cmd: String,
}

#[derive(Debug)]
pub struct JobState {
    jobs: Vec<Job>,
}

impl Default for JobState {
    fn default() -> Self {
        Self::new()
    }
}

impl JobState {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    fn get_next_job_id(&self) -> u32 {
        let mut mask = 0u64;
        for job in self.jobs.iter() {
            mask |= 1u64 << (job.id - 1);
        }
        (!mask).trailing_zeros() + 1
    }

    pub fn push_job(&mut self, child: Child, cmd: String) -> u32 {
        let job_id = self.get_next_job_id();
        let pid = child.id();

        let new_job = Job {
            child,
            status: JobStatus::default(),
            pid,
            id: job_id,
            cmd,
        };
        self.jobs.push(new_job);
        self.reap_finished_jobs();

        println!("[{}]    {}", job_id, pid);
        std::io::stdout().flush().unwrap();

        job_id
    }

    pub fn remove_job(&mut self, job_id: u32) {
        self.jobs.retain(|job| job.id != job_id);
    }

    pub fn get_job(&self, job_id: u32) -> Option<&Job> {
        self.jobs.iter().find(|job| job.id == job_id)
    }

    pub fn update_job_status(&mut self, job_id: u32, status: JobStatus) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
        }
    }

    pub fn format_jobs(&mut self) -> String {
        self.reap_finished_jobs();
        let formatted_jobs = Self::job_formatter(self.jobs.iter());
        self.clear_done_jobs();
        formatted_jobs
    }

    pub fn print_done_job(&mut self) {
        self.reap_finished_jobs();

        let executed_jobs = self
            .jobs
            .iter()
            .filter(|job| job.status != JobStatus::Running);
        let formatted_jobs = Self::job_formatter(executed_jobs);

        print!("{}", formatted_jobs);

        self.clear_done_jobs();
    }

    pub fn clear_done_jobs(&mut self) {
        self.jobs.retain(|job| job.status == JobStatus::Running);
    }

    fn reap_finished_jobs(&mut self) {
        for job in self.jobs.iter_mut() {
            if let Ok(Some(exit_status)) = job.child.try_wait() {
                if exit_status.success() {
                    job.status = JobStatus::Done;
                } else {
                    job.status = JobStatus::Failed(exit_status.code().unwrap_or(1));
                }
            }
        }
    }
    fn job_formatter<'a>(jobs: impl Iterator<Item = &'a Job>) -> String {
        let mut parts = Vec::new();
        for job in jobs {
            let part = format!("[{}]  +/- {}    {}\n", job.id, job.status, job.cmd);
            parts.push(part);
        }

        if !parts.is_empty() {
            parts.push("\n".to_string());
            parts.join("")
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Spawns a real, trivial child process (the POSIX `true` utility, which
    /// exits immediately) purely so `push_job` has an actual `Child` to take
    /// ownership of. These tests exercise the job-id allocation/reuse logic,
    /// not the spawned process's behavior.
    fn spawn_dummy_child() -> Child {
        Command::new("true")
            .spawn()
            .expect("spawn dummy child for test")
    }

    /// Convenience wrapper: pushes a dummy child with a placeholder command
    /// string, since these tests don't care about the display text.
    fn push_dummy_job(state: &mut JobState) -> u32 {
        state.push_job(spawn_dummy_child(), "dummy".to_string())
    }

    #[test]
    fn sequential_jobs_receive_increasing_ids() {
        let mut state = JobState::new();
        let id1 = push_dummy_job(&mut state);
        let id2 = push_dummy_job(&mut state);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn removing_earlier_job_does_not_cause_id_collision_on_subsequent_adds() {
        let mut state = JobState::new();
        let id1 = push_dummy_job(&mut state);
        let id2 = push_dummy_job(&mut state);

        // Job 1 finishes and is removed while Job 2 is still running
        state.remove_job(id1);
        assert!(state.get_job(id1).is_none());
        assert!(state.get_job(id2).is_some());

        // Adding a new job while Job 2 is still active
        let id3 = push_dummy_job(&mut state);
        assert_ne!(
            id3, id2,
            "Newly added job ID must not collide with currently active job ID"
        );
        assert!(state.get_job(id2).is_some());
        assert!(state.get_job(id3).is_some());
    }

    #[test]
    fn lowest_unused_id_reused_when_available() {
        let mut state = JobState::new();
        // Start 3 jobs: IDs 1, 2, 3
        let id1 = push_dummy_job(&mut state);
        let id2 = push_dummy_job(&mut state);
        let id3 = push_dummy_job(&mut state);
        assert_eq!((id1, id2, id3), (1, 2, 3));

        // Remove job 1 -> lowest unused is now 1
        state.remove_job(id1);
        let new_id1 = push_dummy_job(&mut state);
        assert_eq!(new_id1, 1);

        // Remove job 2 -> lowest unused is now 2
        state.remove_job(id2);
        let new_id2 = push_dummy_job(&mut state);
        assert_eq!(new_id2, 2);

        // Next job should get 4
        let id4 = push_dummy_job(&mut state);
        assert_eq!(id4, 4);
    }

    #[test]
    fn multiple_arbitrary_gaps_reused_in_ascending_order() {
        let mut state = JobState::new();
        let mut ids = Vec::new();
        for _ in 0..10 {
            ids.push(push_dummy_job(&mut state));
        }
        // Free slots 3 and 7 (IDs 4 and 8, since 1-indexed)
        state.remove_job(ids[3]);
        state.remove_job(ids[7]);

        // Next two additions should take ID 4 and then ID 8
        let reused_first = push_dummy_job(&mut state);
        let reused_second = push_dummy_job(&mut state);
        assert_eq!(reused_first, ids[3]);
        assert_eq!(reused_second, ids[7]);

        // Next addition should take ID 11
        let next_id = push_dummy_job(&mut state);
        assert_eq!(next_id, 11);
    }
}
