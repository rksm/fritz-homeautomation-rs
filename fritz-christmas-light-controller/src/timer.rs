use chrono::{prelude::*, Duration};
use flume::{Receiver, Sender};

use crate::Interval;

enum TimerConfig {
    Add(DateTime<Local>),
    Replace(Vec<DateTime<Local>>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WaitResult {
    Exit,
    Continue,
}

#[derive(Debug)]
pub struct Timer {
    config_tx: Sender<TimerConfig>,
    timer_rx: Receiver<()>,
}

struct TimerState {
    regular_update: Duration,
    times: Vec<DateTime<Local>>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            regular_update: Duration::seconds(1),
            times: Default::default(),
        }
    }
}

impl Timer {
    pub fn with_regular_update(regular_update: Duration) -> Self {
        let (config_tx, config_rx) = flume::bounded(1);
        let (timer_tx, timer_rx) = flume::bounded(0);

        let _timer_thread = std::thread::spawn(move || {
            let mut timer = TimerState {
                regular_update,
                times: Vec::new(),
            };
            while let WaitResult::Continue = timer.wait(&config_rx, &timer_tx) {}
            debug!("timer thread exiting");
        });

        Self {
            config_tx,
            timer_rx,
        }
    }

    pub fn add_time(&self, t: DateTime<Local>) {
        let _ = self.config_tx.send(TimerConfig::Add(t));
    }

    pub fn replace_times(&self, t: Vec<DateTime<Local>>) {
        let _ = self.config_tx.send(TimerConfig::Replace(t));
    }

    pub fn timer_rx(&self) -> Receiver<()> {
        self.timer_rx.clone()
    }

    pub fn set_intervals(&self, intervals: &[Interval]) {
        self.replace_times(intervals.iter().map(|ea| ea.start).collect());
    }
}

impl TimerState {
    fn wait(&mut self, config_rx: &Receiver<TimerConfig>, timer_tx: &Sender<()>) -> WaitResult {
        self.update_times();

        let wait_timeout = self
            .times
            .first()
            .map(|t| self.regular_update.min(*t - Local::now()))
            .unwrap_or(self.regular_update);

        debug!(
            "waiting until {} ({})",
            Local::now() + wait_timeout,
            crate::duration::duration_pretty(wait_timeout),
        );

        match config_rx.recv_timeout(wait_timeout.to_std().unwrap()) {
            Ok(TimerConfig::Add(val)) => {
                debug!("adding time");
                self.times.push(val);
            }
            Ok(TimerConfig::Replace(items)) => {
                debug!("replacing times");
                self.times = items;
            }
            Err(flume::RecvTimeoutError::Timeout) => {
                if timer_tx.send(()).is_err() {
                    debug!("timer channel closed, exiting");
                    return WaitResult::Exit;
                }
            }
            Err(flume::RecvTimeoutError::Disconnected) => {
                debug!("config channel closed, exiting");
                return WaitResult::Exit;
            }
        }

        WaitResult::Continue
    }

    fn update_times(&mut self) {
        let now = Local::now();
        self.times.retain(|t| *t > now);
        self.times.sort();
        debug!("updated times, waiting for {}", self.times.len());

        if enabled!(tracing::Level::DEBUG) {
            for t in &self.times {
                trace!("  {t} ({})", crate::duration::duration_pretty(*t - now));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tracing_test::traced_test]
    #[test]
    fn test_update_times() {
        let mut timer = TimerState::default();
        let now = Local::now();
        let times = [
            now - Duration::seconds(3),
            now + Duration::seconds(10),
            now + Duration::seconds(3),
        ];

        timer.times.push(times[0]);
        timer.times.push(times[1]);
        timer.times.push(times[2]);
        timer.update_times();

        assert_eq!(timer.times.len(), 2);
        assert_eq!(timer.times[0], times[2]);
        assert_eq!(timer.times[1], times[1]);
    }

    #[tracing_test::traced_test]
    #[test]
    fn timer_scheduling_and_wait() {
        let start = Local::now();
        let timer = Timer::with_regular_update(Duration::milliseconds(300));

        let times = [
            Local::now() + Duration::milliseconds(100),
            Local::now() + Duration::milliseconds(200),
        ];

        timer.replace_times(times.to_vec());

        info!("WAITING FOR TIMER 1");
        timer.timer_rx().recv().expect("wait for timer");
        let delta = Local::now().signed_duration_since(times[0]);
        dbg!((Local::now() - start).num_milliseconds());
        dbg!(delta.num_milliseconds());
        assert!(delta.num_milliseconds().abs() < 10);

        info!("WAITING FOR TIMER 2");
        timer.timer_rx().recv().expect("wait for timer");
        let delta = Local::now().signed_duration_since(times[1]);
        assert!(delta.num_milliseconds().abs() < 10);

        info!("WAITING FOR TIMER 3");
        let start = Local::now();
        timer.timer_rx().recv().expect("wait for timer");
        let delta = Local::now().signed_duration_since(start);
        assert!(delta.num_milliseconds().abs() > 290);
        assert!(delta.num_milliseconds().abs() < 310);
    }
}
