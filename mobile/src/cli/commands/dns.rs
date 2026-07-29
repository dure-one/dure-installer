//! DNS command implementation

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::db;
use crate::calc::dns::{RecordType, resolve_dns};
use crate::storage::models::dns::{cache_dns_records, get_cached_dns_records, init_dns_table};
use anyhow::Result;
use diesel::prelude::*;

/// Execute DNS command
pub fn execute_dns(record_type_str: &str, domain: &str) -> Result<()> {
    let record_type = match record_type_str.to_lowercase().as_str() {
        "a" => RecordType::A,
        "aaaa" => RecordType::AAAA,
        "txt" => RecordType::TXT,
        "ns" => RecordType::NS,
        _ => anyhow::bail!("Invalid record type. Use: a, aaaa, ns, or txt"),
    };

    // Get database connection
    let mut conn = db::establish_connection();

    // Initialize table if needed
    init_dns_table(&mut conn)?;

    // Try cache first
    let records = {
        let cached = get_cached_dns_records(&mut conn, domain, record_type)?;
        if !cached.is_empty() {
            dure_info!("Using cached results for {domain} {record_type}");
            cached
        } else {
            // Cache miss, fetch fresh
            fetch_and_cache(&mut conn, domain, record_type)?
        }
    };

    // Display results
    if records.is_empty() {
        println!("No {} records found for {}", record_type, domain);
    } else {
        println!("{} records for {}:", record_type, domain);
        for record in records {
            println!("  {} (TTL: {}s)", record.value, record.ttl);
        }
    }

    Ok(())
}

fn fetch_and_cache(
    conn: &mut SqliteConnection,
    domain: &str,
    record_type: RecordType,
) -> Result<Vec<crate::calc::dns::DnsRecord>> {
    dure_info!("Fetching fresh DNS records for {domain} {record_type}...");
    let records = resolve_dns(domain, record_type)?;

    if !records.is_empty() {
        cache_dns_records(conn, &records)?;
    }

    Ok(records)
}

fn get_db_path() -> Result<std::path::PathBuf> {
    let db_path = crate::calc::db::get_db_path();
    Ok(std::path::PathBuf::from(db_path))
}

/// Execute DNS bastion command - add TXT record for bastion IP
pub fn execute_dns_bastion(ip: &str) -> Result<()> {
    // Validate IP address format
    if !is_valid_ip(ip) {
        anyhow::bail!("Invalid IP address format: {}", ip);
    }

    let mut conn = db::establish_connection();
    init_dns_table(&mut conn)?;

    // Create a TXT record for bastion IP with domain "bastion.local"
    let domain = "bastion.local";
    let txt_value = format!("bastion-ip={}", ip);

    let record = crate::calc::dns::DnsRecord::new(
        domain.to_string(),
        RecordType::TXT,
        txt_value.clone(),
        86400, // 24 hour TTL
    );

    cache_dns_records(&mut conn, &[record])?;

    println!("✓ Added bastion IP to DNS cache:");
    println!("  Domain: {}", domain);
    println!("  Type: TXT");
    println!("  Value: {}", txt_value);
    println!("  IP: {}", ip);

    Ok(())
}

/// Validate IP address format (simple IPv4/IPv6 check)
fn is_valid_ip(ip: &str) -> bool {
    // Simple validation - check for IPv4 or IPv6 format
    ip.parse::<std::net::IpAddr>().is_ok()
}
