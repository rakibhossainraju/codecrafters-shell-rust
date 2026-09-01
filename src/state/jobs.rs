pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

pub struct Job {
    status: JobStatus,
    pid: i32,
    id: i32,
}

pub struct JobState {
    jobs: Vec<Job>,
    next_job_id: i32,
}

impl JobState {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_job_id: 1,
        }
    }

    pub fn add_job(&mut self, status: JobStatus, pid: i32) -> i32 {
        let job_id = self.next_job_id;
        self.next_job_id += 1;
        self.jobs.push(Job {
            status,
            pid,
            id: job_id,
        });
        job_id
    }

    pub fn remove_job(&mut self, job_id: i32) {
        self.jobs.retain(|job| job.id != job_id);
    }

    pub fn get_job(&self, job_id: i32) -> Option<&Job> {
        self.jobs.iter().find(|job| job.id == job_id)
    }

    pub fn update_job_status(&mut self, job_id: i32, status: JobStatus) {
        if let Some(job) = self.jobs.iter_mut().find(|job| job.id == job_id) {
            job.status = status;
        }
    }
}
