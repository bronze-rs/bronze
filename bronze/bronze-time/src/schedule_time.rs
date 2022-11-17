use crate::prelude::ScheduleExpr;
use bronze_utils::{debug, BronzeError};
use chrono::{DateTime, Duration, Local, Utc};
use cron::ScheduleIterator;
use std::cmp::Ordering;
use std::iter::Take;
use std::str::FromStr;

type InternalDateTime = DateTime<Utc>;

#[allow(dead_code)]
const DEFAULT_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

#[derive(Debug, Clone)]
pub struct ScheduleTime {
    pub(crate) dt: InternalDateTime,
}

#[derive(Debug, Clone)]
pub struct ScheduleTimeHolder {
    pub(crate) expr: ScheduleExpr,
    pub(crate) min_interval: Option<Duration>,
    pub(crate) last_run: Option<ScheduleTime>,
    pub(crate) next_run: Option<ScheduleTime>,
}

pub trait ScheduleTimeOp {
    fn last_run(&self) -> Option<ScheduleTime>;

    fn next_run(&self) -> Option<ScheduleTime>;

    fn set_last_run(&mut self, t: &ScheduleTime) -> &mut Self;

    fn set_next_run(&mut self, t: &ScheduleTime) -> &mut Self;
}

impl ScheduleTime {
    pub fn new(dt: InternalDateTime) -> Self {
        ScheduleTime { dt }
    }
    pub fn from_now() -> Self {
        let local_time = Local::now();
        let utc_time = DateTime::<Utc>::from_utc(local_time.naive_utc(), Utc);
        ScheduleTime::new(utc_time)
    }
}

impl From<InternalDateTime> for ScheduleTime {
    fn from(dt: InternalDateTime) -> Self {
        ScheduleTime::new(dt)
    }
}

impl FromStr for ScheduleTime {
    type Err = BronzeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InternalDateTime::from_str(s)
            .map(InternalDateTime::into)
            .map_err(BronzeError::new)
    }
}

impl PartialEq<Self> for ScheduleTime {
    fn eq(&self, other: &Self) -> bool {
        self.dt == other.dt
    }
}

impl PartialOrd for ScheduleTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.dt.partial_cmp(&other.dt)
    }
}

impl ScheduleTimeHolder {
    pub fn new(expr: ScheduleExpr) -> Self {
        ScheduleTimeHolder {
            expr,
            min_interval: None,
            last_run: None,
            next_run: None,
        }
    }

    pub fn init(&mut self) {
        let now = ScheduleTime::from_now();
        match self.expr.clone() {
            ScheduleExpr::Cron(c) => {
                let mut times = c.after(&now.dt).take(21);
                debug!("Now: {}", now.dt);
                let next_run = times.next().unwrap();

                // Get the minimum time interval from the 20 schedule times
                let min_interval: Duration = times
                    .collect::<Vec<InternalDateTime>>()
                    .windows(2)
                    .map(|x| x[1] - x[0])
                    .min()
                    .unwrap();
                debug!("min interval is: {}", min_interval.num_seconds());
                self.min_interval = Some(min_interval);
                self.next_run = Some(ScheduleTime::from(next_run));
            },
            _ => panic!(),
        }
    }

    fn get_next_times(
        &self,
        n: usize,
        from: &InternalDateTime,
    ) -> Option<Take<ScheduleIterator<'_, Utc>>> {
        match &self.expr {
            ScheduleExpr::Cron(c) => Some(c.after(from).take(n)),
            ScheduleExpr::Preset(_) => None,
        }
    }

    pub fn cmp_and_to_next(&mut self, from: &ScheduleTime) -> bool {
        if let Some(ref next_time) = self.next_run {
            if from.dt >= next_time.dt {
                debug!("from: {}, next_run: {}", from.dt, &next_time.dt);
                let copy_next_run = next_time.clone();
                self.last_run = Some(copy_next_run);
                if let Some(mut it) = self.get_next_times(1, &next_time.dt) {
                    let new_time = it.next().unwrap();
                    debug!("set new time {}", new_time);
                    self.next_run = Some(ScheduleTime::from(new_time));
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::ScheduleExpr;
    use crate::schedule_time::ScheduleTimeHolder;
    use std::str::FromStr;

    #[test]
    fn test_time_holder() {
        let expr = ScheduleExpr::from_str("@daily").unwrap();

        let mut s = ScheduleTimeHolder::new(expr);
        s.init()
    }
}
