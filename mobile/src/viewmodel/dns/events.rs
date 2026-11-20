//! DNS actor events

use super::commands::RecordType;

#[derive(Debug, Clone)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub name_servers: Vec<String>,
    pub account_id: String,
    pub account_name: String,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub record_type: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Clone)]
pub struct SrvRecord {
    pub id: String,
    pub name: String,
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum DnsEvent {
    // OAuth
    AccessTokenSet,

    // Zone Events
    ZonesListed {
        zones: Vec<Zone>,
    },

    ZoneFound {
        zone: Option<Zone>,
    },

    ZoneCreated {
        zone: Zone,
    },

    // Record Events
    RecordsListed {
        zone_id: String,
        records: Vec<DnsRecord>,
    },

    RecordFound {
        zone_id: String,
        record: Option<DnsRecord>,
    },

    RecordCreated {
        zone_id: String,
        name: String,
        record_type: String,
    },

    RecordUpdated {
        zone_id: String,
        record_id: String,
    },

    RecordDeleted {
        zone_id: String,
        record_id: String,
    },

    RecordUpserted {
        zone_id: String,
        name: String,
        record_type: String,
    },

    // SRV Record Events
    SrvRecordCreated {
        zone_id: String,
        name: String,
    },

    SrvRecordUpserted {
        zone_id: String,
        name: String,
    },

    // Preset Operations
    DefaultXmppRecordsCreated {
        zone_id: String,
        domain: String,
    },

    DefaultXmppRecordsVerified {
        zone_id: String,
        domain: String,
        verified: bool,
    },

    Error {
        operation: String,
        error: String,
    },
}
