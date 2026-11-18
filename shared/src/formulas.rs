pub fn calculate_daily_hours(standard_hours_per_day: f64, avg_employees_present: f64) -> f64 {
    standard_hours_per_day * avg_employees_present
}

pub fn calculate_monthly_cumulative_hours(
    effective_days: f64,
    standard_hours_per_day: f64,
    avg_employees_present: f64,
) -> f64 {
    effective_days * calculate_daily_hours(standard_hours_per_day, avg_employees_present)
}

/// Frequency Rate (FR) = (Jumlah LTI * 1.000.000) / Total Jam Kerja
pub fn calculate_fr(lti: f64, safe_manhours: f64) -> f64 {
    if safe_manhours <= 0.0 {
        0.0
    } else {
        (lti * 1_000_000.0) / safe_manhours
    }
}

/// Severity Rate (SR) = (Jumlah Hari Hilang * 1.000.000) / Total Jam Kerja
pub fn calculate_sr(lost_days: f64, safe_manhours: f64) -> f64 {
    if safe_manhours <= 0.0 {
        0.0
    } else {
        (lost_days * 1_000_000.0) / safe_manhours
    }
}

/// Total Recordable Incident Rate (TRIR) = ((Fatality + LTI + RWC + MTC) * 200.000) / Total Jam Kerja
pub fn calculate_trir(fatality: f64, lti: f64, rwc: f64, mtc: f64, safe_manhours: f64) -> f64 {
    if safe_manhours <= 0.0 {
        0.0
    } else {
        ((fatality + lti + rwc + mtc) * 200_000.0) / safe_manhours
    }
}

/// Capaian (%) = IFERROR(Realisasi / Target, 0) * 100
pub fn calculate_achievement_pct(realization: f64, target: f64) -> f64 {
    if target <= 0.0 {
        0.0
    } else {
        (realization / target) * 100.0
    }
}

pub fn month_short_name(month: i32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "Mei",
        6 => "Jun",
        7 => "Jul",
        8 => "Agt",
        9 => "Sep",
        10 => "Okt",
        11 => "Nov",
        12 => "Des",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_manhours_and_rates() {
        let m = calculate_monthly_cumulative_hours(19.0, 8.0, 100.0);
        assert_eq!(m, 15200.0);
        assert_eq!(calculate_fr(0.0, 15200.0), 0.0);
        assert_eq!(calculate_fr(1.0, 0.0), 0.0);
        assert!((calculate_fr(1.0, 1_000_000.0) - 1.0).abs() < 1e-6);
        assert!((calculate_sr(5.0, 1_000_000.0) - 5.0).abs() < 1e-6);
        assert!((calculate_trir(0.0, 1.0, 1.0, 0.0, 200_000.0) - 2.0).abs() < 1e-6);
        assert_eq!(calculate_achievement_pct(6.0, 12.0), 50.0);
        assert_eq!(calculate_achievement_pct(5.0, 0.0), 0.0);
    }
}
