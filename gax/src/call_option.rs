use rand::Rng;
use tonic::{Code, Status};

pub trait Retryer {
    fn retry(&mut self, err: &tonic::Status) -> (std::time::Duration, bool);
}

// The wait time between retries is a random value between 0 and the "retry envelope".
// The envelope starts at Initial and increases by the factor of Multiplier every retry,
// but is capped at Max.
#[derive(Clone)]
pub struct Backoff {
    pub initial: chrono::Duration,
    pub max: chrono::Duration,
    pub multiplier: f64,
    pub cur: chrono::Duration,
}

impl Backoff {
    fn pause(&mut self) -> chrono::Duration {
        if self.initial.is_zero() {
            self.initial = chrono::Duration::seconds(1);
        }
        if self.cur.is_zero() {
            self.cur = self.initial;
        }
        if self.max.is_zero() {
            self.max = chrono::Duration::seconds(30);
        }
        if self.multiplier < 1.0 {
            self.multiplier = 2.0;
        }

        // Select a duration between 1ns and the current max. It might seem
        // counterintuitive to have so much jitter, but
        // https://www.awsarchitectureblog.com/2015/03/backoff.html argues that
        // that is the best strategy.
        let mut rng = rand::thread_rng();
        let cur_val = self.cur.num_nanoseconds().unwrap();
        let d = chrono::Duration::nanoseconds(1 + rng.gen_range(0..cur_val));
        self.cur = chrono::Duration::nanoseconds((cur_val as f64 * self.multiplier) as i64);
        if self.cur > self.max {
            self.cur = self.max;
        }
        return d;
    }
}

#[derive(Clone)]
pub struct BackoffRetryer {
    pub backoff: Backoff,
    pub codes: Vec<tonic::Code>,
    pub check_session_not_found: bool,
}

impl Default for Backoff {
    fn default() -> Self {
        Backoff {
            initial: chrono::Duration::microseconds(250),
            max: chrono::Duration::microseconds(32000),
            multiplier: 1.30,
            cur: chrono::Duration::zero(),
        }
    }
}

#[derive(Clone)]
pub struct CallSettings {
    pub retryer: BackoffRetryer,
}

impl BackoffRetryer {
    pub fn retry(&mut self, status: &Status) -> (chrono::Duration, bool) {
        let code = status.code();
        if code == Code::Internal
            && !status.message().contains("stream terminated by RST_STREAM")
            && !status
                .message()
                .contains("HTTP/2 error code: INTERNAL_ERROR")
            && !status
                .message()
                .contains("Connection closed with unknown cause")
            && !status
                .message()
                .contains("Received unexpected EOS on DATA frame from server")
        {
            return (chrono::Duration::nanoseconds(0), false);
        }

        for candidate in self.codes.iter() {
            if *candidate == code {
                log::debug!("retry {} {}", status.code(), status.message());
                return (self.backoff.pause(), true);
            }
        }

        if self.check_session_not_found {
            if status.message().contains("Session not found:") {
                log::debug!("retry by session not found");
                return (self.backoff.pause(), true);
            }
        }
        return (chrono::Duration::nanoseconds(0), false);
    }
}
