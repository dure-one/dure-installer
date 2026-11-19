//! AUDIT command implementation
//!
//! `dure audit status` - display recent audit records
//! `dure audit clear`  - wipe all audit records

use crate::calc::audit;
use crate::calc::profile::ProfileContext;
use anyhow::Result;

/// Execute `audit status` — print recent audit records to stdout.
pub fn execute_audit_status(profile: &ProfileContext) -> Result<()> {
    let records = audit::show_for_profile(profile, 50)?;

    if records.is_empty() {
        println!("No audit records found.");
        return Ok(());
    }

    println!(
        "{:<6} {:<20} {:<14} {:<16} {:<8} {:<24} {:<16} {:<8}",
        "ID", "Timestamp", "Category", "Actor", "Surface", "Action", "Object", "Outcome"
    );
    println!("{}", "-".repeat(120));

    for r in &records {
        // Format unix timestamp as human-readable UTC
        let ts = format_ts(r.timestamp);
        println!(
            "{:<6} {:<20} {:<14} {:<16} {:<8} {:<24} {:<16} {:<8}",
            r.id, ts, r.category, r.actor_id, r.surface, r.action, r.object, r.outcome,
        );
        if !r.detail.is_empty() {
            println!("       detail: {}", r.detail);
        }
    }

    Ok(())
}

/// Execute `audit clear` — delete all audit records after confirmation.
pub fn execute_audit_clear(profile: &ProfileContext) -> Result<()> {
    use std::io::{self, BufRead};

    print!("This will permanently delete all audit records. Type 'yes' to confirm: ");
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    if line.trim().eq_ignore_ascii_case("yes") {
        let deleted = audit::clear_for_profile(profile)?;
        println!("Cleared {} audit record(s).", deleted);
    } else {
        println!("Aborted.");
    }

    Ok(())
}

fn format_ts(unix_secs: i64) -> String {
    // Simple ISO-8601-ish formatting without external crates
    if unix_secs == 0 {
        return "unknown".to_string();
    }
    let secs = unix_secs as u64;
    // Days since 1970-01-01
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;

    // Approximate calendar date (no timezone awareness, no leap second)
    let (y, mo, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, m, s)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let ydays = if leap { 366 } else { 365 };
        if days < ydays {
            break;
        }
        days -= ydays;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
