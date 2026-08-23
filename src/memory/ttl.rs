//! Memory states: active → warm → cold → archive. Nothing is ever deleted.
//! D1: classification only (states are derived from age at read time).

#[derive(PartialEq, Debug)]
pub enum TtlState {
    Active,
    Warm,
    Cold,
    Archive,
}

/// Ages in days after which a record leaves its current state.
pub const WARM_AFTER_DAYS: i64 = 90;
pub const COLD_AFTER_WARM_DAYS: i64 = 90;
pub const ARCHIVE_AFTER_COLD_DAYS: i64 = 275;

pub fn classify(age_days: i64) -> TtlState {
    if age_days < WARM_AFTER_DAYS {
        TtlState::Active
    } else if age_days < WARM_AFTER_DAYS + COLD_AFTER_WARM_DAYS {
        TtlState::Warm
    } else if age_days < WARM_AFTER_DAYS + COLD_AFTER_WARM_DAYS + ARCHIVE_AFTER_COLD_DAYS {
        TtlState::Cold
    } else {
        TtlState::Archive
    }
}

/// Count records per state given their ages in days.
pub fn sweep(ages_days: &[i64]) -> std::collections::HashMap<String, usize> {
    let mut out = std::collections::HashMap::new();
    for a in ages_days {
        let name = match classify(*a) {
            TtlState::Active => "active",
            TtlState::Warm => "warm",
            TtlState::Cold => "cold",
            TtlState::Archive => "archive",
        };
        *out.entry(name.to_string()).or_insert(0) += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_progress_and_never_delete() {
        assert_eq!(classify(0), TtlState::Active);
        assert_eq!(classify(89), TtlState::Active);
        assert_eq!(classify(91), TtlState::Warm);
        assert_eq!(classify(200), TtlState::Cold);
        assert_eq!(classify(1000), TtlState::Archive);
    }
}
