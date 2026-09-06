use std::process::Child;

#[derive(Debug, Default)]
pub enum JobStatus {
    #[default]
    Running,
    Completed,
    Failed,
}

#[derive(Debug)]
pub struct Job {
    status: JobStatus,
    pid: u32,
    id: u32,
    child: Child,
}

#[derive(Debug)]
pub struct JobState {
    jobs: Vec<Job>,
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
        let next_id = (!mask).trailing_zeros() as u32 + 1;
        next_id
    }

    pub fn push_job(&mut self, child: Child) -> u32 {
        let job_id = self.get_next_job_id();
        let pid = child.id();

        let new_job = Job {
            child,
            status: JobStatus::default(),
            pid,
            id: job_id,
        };
        self.jobs.push(new_job);
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

    #[test]
    fn sequential_jobs_receive_increasing_ids() {
        let mut state = JobState::new();
        let id1 = state.push_job(spawn_dummy_child());
        let id2 = state.push_job(spawn_dummy_child());
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn removing_earlier_job_does_not_cause_id_collision_on_subsequent_adds() {
        let mut state = JobState::new();
        let id1 = state.push_job(spawn_dummy_child());
        let id2 = state.push_job(spawn_dummy_child());

        // Job 1 finishes and is removed while Job 2 is still running
        state.remove_job(id1);
        assert!(state.get_job(id1).is_none());
        assert!(state.get_job(id2).is_some());

        // Adding a new job while Job 2 is still active
        let id3 = state.push_job(spawn_dummy_child());
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
        let id1 = state.push_job(spawn_dummy_child());
        let id2 = state.push_job(spawn_dummy_child());
        let id3 = state.push_job(spawn_dummy_child());
        assert_eq!((id1, id2, id3), (1, 2, 3));

        // Remove job 1 -> lowest unused is now 1
        state.remove_job(id1);
        let new_id1 = state.push_job(spawn_dummy_child());
        assert_eq!(new_id1, 1);

        // Remove job 2 -> lowest unused is now 2
        state.remove_job(id2);
        let new_id2 = state.push_job(spawn_dummy_child());
        assert_eq!(new_id2, 2);

        // Next job should get 4
        let id4 = state.push_job(spawn_dummy_child());
        assert_eq!(id4, 4);
    }

    #[test]
    fn multiple_arbitrary_gaps_reused_in_ascending_order() {
        let mut state = JobState::new();
        let mut ids = Vec::new();
        for _ in 0..10 {
            ids.push(state.push_job(spawn_dummy_child()));
        }
        // Free slots 3 and 7 (IDs 4 and 8, since 1-indexed)
        state.remove_job(ids[3]);
        state.remove_job(ids[7]);

        // Next two additions should take ID 4 and then ID 8
        let reused_first = state.push_job(spawn_dummy_child());
        let reused_second = state.push_job(spawn_dummy_child());
        assert_eq!(reused_first, ids[3]);
        assert_eq!(reused_second, ids[7]);

        // Next addition should take ID 11
        let next_id = state.push_job(spawn_dummy_child());
        assert_eq!(next_id, 11);
    }
}
