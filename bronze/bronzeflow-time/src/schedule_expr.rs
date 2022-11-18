use bronzeflow_utils::ayn_error;
use chrono::Duration;
use cron::Schedule;
use std::str::FromStr;

use bronzeflow_utils::prelude::{BronzeError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FixPreset {
    /// The task does not need to run
    NotRun,
    /// The task is run only once
    Once,
    /// The task runs every hour
    Hourly,
    /// The task runs every day
    Daily,
    /// The task runs one a week
    Weekly,
    /// The task runs every month
    Monthly,
    /// The task runs every year
    Yearly,
}

#[derive(Debug, Clone)]
pub enum SchedulePreset {
    NotRun,
    Once,
}

// TODO, support duration schedule
#[derive(Debug, Clone)]
pub struct ScheduleDuration(Duration);

#[derive(Debug, Clone)]
pub enum ScheduleExpr {
    Cron(Schedule),
    Preset(SchedulePreset),
    // Duration(ScheduleDuration)
}

impl ScheduleExpr {
    pub fn to_cron_schedule(&self) -> Result<Schedule> {
        match self {
            ScheduleExpr::Cron(s) => Ok(s.clone()),
            _ => Err(BronzeError::msg("Can`t transform to schedule")),
        }
    }
}

impl<'a> TryFrom<&'a str> for ScheduleExpr {
    type Error = BronzeError;

    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        ScheduleExpr::from_str(value)
    }
}

impl TryFrom<FixPreset> for ScheduleExpr {
    type Error = BronzeError;

    fn try_from(value: FixPreset) -> std::result::Result<Self, Self::Error> {
        match value {
            FixPreset::NotRun => Ok(ScheduleExpr::Preset(SchedulePreset::NotRun)),
            FixPreset::Once => Ok(ScheduleExpr::Preset(SchedulePreset::Once)),
            s => {
                let s: String = s.get_string();
                Schedule::from_str(s.as_str())
                    .map(ScheduleExpr::Cron)
                    .map_err(BronzeError::msg)
            },
        }
    }
}

impl FromStr for FixPreset {
    type Err = BronzeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim() {
            "@none" => Ok(FixPreset::NotRun),
            "@once" => Ok(FixPreset::Once),
            "@hourly" => Ok(FixPreset::Hourly),
            "@daily" => Ok(FixPreset::Daily),
            "@weekly" => Ok(FixPreset::Weekly),
            "@monthly" => Ok(FixPreset::Monthly),
            "@yearly" => Ok(FixPreset::Yearly),
            _ => Err(ayn_error!("Error preset string: {}", s)),
        }
    }
}

impl FromStr for ScheduleExpr {
    type Err = BronzeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.trim();
        if let Ok(c) = Schedule::from_str(s) {
            return Ok(ScheduleExpr::Cron(c));
        }
        FixPreset::from_str(s).map_or_else(Err, |r: FixPreset| r.try_into())
    }
}

impl FixPreset {
    fn get_string(&self) -> String {
        match self {
            FixPreset::NotRun => "@none".to_string(),
            FixPreset::Once => "@once".to_string(),
            FixPreset::Hourly => "@hourly".to_string(),
            FixPreset::Daily => "@daily".to_string(),
            FixPreset::Weekly => "@weekly".to_string(),
            FixPreset::Monthly => "@monthly".to_string(),
            FixPreset::Yearly => "@yearly".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches::assert_matches;

    #[test]
    fn test_parse_fix_reset_from_str() {
        assert_eq!(FixPreset::NotRun, FixPreset::from_str("@none").unwrap());
        assert_eq!(FixPreset::Once, FixPreset::from_str("@once").unwrap());
        assert_eq!(FixPreset::Hourly, FixPreset::from_str("@hourly").unwrap());
        assert_eq!(FixPreset::Daily, FixPreset::from_str("@daily").unwrap());
        assert_eq!(FixPreset::Weekly, "@weekly".parse().unwrap());
        assert_eq!(FixPreset::Monthly, "@monthly".parse().unwrap());
        assert_eq!(FixPreset::Yearly, "@yearly".parse().unwrap());
    }

    #[test]
    fn test_parse_schedule_expr_from_str() {
        let not_run = ScheduleExpr::from_str("@none").ok();
        assert_matches!(not_run, Some(ScheduleExpr::Preset(SchedulePreset::NotRun)));

        let once = ScheduleExpr::from_str("@once").ok();
        assert_matches!(once, Some(ScheduleExpr::Preset(SchedulePreset::Once)));

        assert_matches!(ScheduleExpr::from_str("error str expr").ok(), None);
        let s1: Option<ScheduleExpr> = "0 0 0 * * 1 *".parse().ok();
        assert_matches!(s1, Some(_));

        let s2 = "1/10 * * * * * *".parse().ok();
        assert_matches!(s2, Some(ScheduleExpr::Cron(_)));
    }
}
