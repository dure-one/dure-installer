//! Audit business logic - transaction recording and query helpers
//!
//! Every user input on CLI / GUI / WASM goes through `push()` to create an
//! immutable audit trail:
//!
//!   "Who, What, Where, and When"
//!   Authentication Data (login attempts, failures, security events)
//!   Event Details (timestamps, IP addresses, actor IDs, device IDs, object affected)
//!   Changes in Privilege (identity switches, elevated privilege grants)

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::db;
use crate::storage::models::audit::{
    AuditCategory, AuditEvent, AuditOutcome, AuditRecord, clear_all, init_audit_table, list_recent,
    record,
};
use anyhow::Result;

/// Record a user action originating from any surface (cli / gui / wasm).
///
/// Returns the newly assigned row id.
pub fn push(
    surface: &str,
    actor_id: &str,
    device_id: &str,
    action: &str,
    object: &str,
    outcome: AuditOutcome,
    detail: &str,
    ip_address: &str,
) -> Result<i64> {
    let mut conn = db::establish_connection();
    init_audit_table(&mut conn)?;
    let event = AuditEvent::new(action)
        .surface(surface)
        .actor(actor_id)
        .device(device_id)
        .object(object)
        .outcome(outcome)
        .detail(detail)
        .ip(ip_address);
    record(&mut conn, event)
}

/// Record a CLI user action with default success outcome.
pub fn push_cli(actor_id: &str, device_id: &str, action: &str, object: &str) -> Result<i64> {
    push(
        "cli",
        actor_id,
        device_id,
        action,
        object,
        AuditOutcome::Success,
        "",
        "",
    )
}

/// Record a GUI user action with default success outcome.
pub fn push_gui(actor_id: &str, device_id: &str, action: &str, object: &str) -> Result<i64> {
    push(
        "gui",
        actor_id,
        device_id,
        action,
        object,
        AuditOutcome::Success,
        "",
        "",
    )
}

/// Record an authentication event.
pub fn push_auth(
    actor_id: &str,
    device_id: &str,
    action: &str,
    outcome: AuditOutcome,
    detail: &str,
    ip_address: &str,
) -> Result<i64> {
    let mut conn = db::establish_connection();
    init_audit_table(&mut conn)?;
    let event = AuditEvent::new(action)
        .category(AuditCategory::Auth)
        .surface("system")
        .actor(actor_id)
        .device(device_id)
        .outcome(outcome)
        .detail(detail)
        .ip(ip_address);
    record(&mut conn, event)
}

/// Record a privilege change event.
pub fn push_privilege_change(
    actor_id: &str,
    device_id: &str,
    action: &str,
    detail: &str,
) -> Result<i64> {
    let mut conn = db::establish_connection();
    init_audit_table(&mut conn)?;
    let event = AuditEvent::new(action)
        .category(AuditCategory::PrivilegeChange)
        .surface("system")
        .actor(actor_id)
        .device(device_id)
        .detail(detail);
    record(&mut conn, event)
}

/// Retrieve the most recent `limit` audit records.
pub fn show(limit: i64) -> Result<Vec<AuditRecord>> {
    let mut conn = db::establish_connection();
    init_audit_table(&mut conn)?;
    list_recent(&mut conn, limit)
}

/// Wipe all audit records (requires explicit user confirmation at call site).
pub fn clear() -> Result<usize> {
    let mut conn = db::establish_connection();
    init_audit_table(&mut conn)?;
    clear_all(&mut conn)
}
