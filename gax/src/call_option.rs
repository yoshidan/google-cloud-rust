use rand::Rng;
use std::time::{Duration, Instant};
use tonic::Status;

/// Backoff implements exponential backoff.
/// The wait time between retries is a random value between 0 and the "retry envelope".
/// The envelope starts at Initial and increases by the factor of Multiplier every retry,
/// but is capped at Max.
#[derive(Clone)]
pub struct Backoff {
    pub initial: Duration,
    pub max: Duration,
    pub multiplier: f64,
    pub timeout: Duration,

    cur: Duration,
    elapsed: Option<Instant>,
}

impl Backoff {
    pub fn duration(&mut self) -> Option<Duration> {
        if self.initial.as_nanos() == 0 {
            self.initial = Duration::from_secs(1);
        }
        if self.cur.as_nanos() == 0 {
            self.cur = self.initial;
        }
        if self.max.as_nanos() == 0 {
            self.max = Duration::from_secs(30)
        }
        if self.multiplier < 1.0 {
            self.multiplier = 2.0
        }
        match self.elapsed {
            None => self.elapsed = Some(Instant::now()),
            Some(s) => if s.elapsed() > self.timeout {
               return None
            }
        };
        // Select a duration between 1ns and the current max. It might seem
        // counterintuitive to have so much jitter, but
        // https://www.awsarchitectureblog.com/2015/03/backoff.html argues that
        // that is the best strategy.
        let mut rng = rand::thread_rng();
        let cur_val = self.cur.as_nanos();
        let d = Duration::from_nanos((1 + rng.gen_range(0..cur_val)) as u64);
        self.cur = Duration::from_nanos((cur_val as f64 * self.multiplier) as u64);
        if self.cur > self.max {
            self.cur = self.max;
        }
        return Some(d);
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Backoff {
            elapsed: None,
            initial: Duration::from_millis(250),
            max: Duration::from_millis(32000),
            multiplier: 1.30,
            cur: Duration::from_nanos(0),
            timeout: Duration::from_millis(32000),
        }
    }
}

/// Retryer is used by Invoke to determine retry behavior.
pub trait Retryer {
    fn retry(&mut self, status: &Status) -> Option<Duration>;
}

#[derive(Clone)]
pub struct BackoffRetryer {
    pub backoff: Backoff, // supports backoff retry only
    pub codes: Vec<tonic::Code>,
}

impl Retryer for BackoffRetryer {
    fn retry(&mut self, status: &Status) -> Option<Duration> {
        let code = status.code();
        for candidate in self.codes.iter() {
            if *candidate == code {
                return self.backoff.duration();
            }
        }
        return None;
    }
}

#[derive(Clone)]
pub struct RetrySettings<T>
where
    T: Retryer + Clone,
{
    pub retryer: T,
}

pub type BackoffRetrySettings = RetrySettings<BackoffRetryer>;
