use crate::{DoweError, DoweResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronSchedule {
    fields: [CronField; 5],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CronField {
    allowed: Vec<bool>,
}

impl CronSchedule {
    pub fn parse(expression: &str) -> DoweResult<Self> {
        let values = expression.split_whitespace().collect::<Vec<_>>();
        if values.len() != 5 {
            return Err(DoweError::new("cron schedule must contain five fields"));
        }
        Ok(Self {
            fields: [
                CronField::parse(values[0], 0, 59, false)?,
                CronField::parse(values[1], 0, 23, false)?,
                CronField::parse(values[2], 1, 31, false)?,
                CronField::parse(values[3], 1, 12, false)?,
                CronField::parse(values[4], 0, 7, true)?,
            ],
        })
    }

    pub fn matches_unix_minute(&self, unix_minute: i64) -> bool {
        let seconds = unix_minute.saturating_mul(60);
        let days = seconds.div_euclid(86_400);
        let seconds_of_day = seconds.rem_euclid(86_400);
        let minute = (seconds_of_day / 60 % 60) as usize;
        let hour = (seconds_of_day / 3_600) as usize;
        let (_, month, day) = civil_from_days(days);
        let weekday = (days + 4).rem_euclid(7) as usize;
        self.fields[0].allows(minute)
            && self.fields[1].allows(hour)
            && self.fields[2].allows(day as usize)
            && self.fields[3].allows(month as usize)
            && self.fields[4].allows(weekday)
    }
}

impl CronField {
    fn parse(source: &str, min: usize, max: usize, sunday_alias: bool) -> DoweResult<Self> {
        let mut allowed = vec![false; max + 1];
        for part in source.split(',') {
            if part.is_empty() {
                return Err(DoweError::new("cron field contains an empty list item"));
            }
            let (range, step) = match part.split_once('/') {
                Some((range, step)) => (range, parse_number(step, 1, max)?),
                None => (part, 1),
            };
            let (start, end) = if range == "*" {
                (min, max)
            } else if let Some((start, end)) = range.split_once('-') {
                (parse_number(start, min, max)?, parse_number(end, min, max)?)
            } else {
                let value = parse_number(range, min, max)?;
                (value, value)
            };
            if start > end {
                return Err(DoweError::new("cron range start must not exceed its end"));
            }
            for value in (start..=end).step_by(step) {
                allowed[value] = true;
            }
        }
        if sunday_alias && allowed[7] {
            allowed[0] = true;
        }
        Ok(Self { allowed })
    }

    fn allows(&self, value: usize) -> bool {
        self.allowed.get(value).copied().unwrap_or(false)
    }
}

fn parse_number(source: &str, min: usize, max: usize) -> DoweResult<usize> {
    let value = source
        .parse::<usize>()
        .map_err(|_| DoweError::new(format!("invalid cron value `{source}`")))?;
    if value < min || value > max {
        return Err(DoweError::new(format!(
            "cron value `{value}` must be between {min} and {max}"
        )));
    }
    Ok(value)
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::CronSchedule;

    #[test]
    fn parses_ranges_lists_steps_and_sunday_alias() {
        CronSchedule::parse("*/15 1-5 * 1,6 0,7").expect("schedule");
    }

    #[test]
    fn matches_utc_minutes() {
        let schedule = CronSchedule::parse("5 3 2 1 *").expect("schedule");
        assert!(schedule.matches_unix_minute(27_877_145));
        assert!(!schedule.matches_unix_minute(27_877_144));
    }

    #[test]
    fn rejects_invalid_schedules() {
        assert!(CronSchedule::parse("* * * *").is_err());
        assert!(CronSchedule::parse("60 * * * *").is_err());
        assert!(CronSchedule::parse("*/0 * * * *").is_err());
    }
}
