//! DNS actor commands

#[derive(Debug, Clone)]
pub enum DnsCommand {
    // OAuth
    SetAccessToken {
        access_token: String,
        refresh_token: Option<String>,
    },

    // Zone Operations
    ListZones,

    FindZone {
        domain: String,
    },

    CreateZone {
        domain: String,
        account_id: String,
    },

    // DNS Record Operations
    ListRecords {
        zone_id: String,
    },

    FindRecord {
        zone_id: String,
        record_type: RecordType,
        name: String,
    },

    CreateRecord {
        zone_id: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    },

    UpdateRecord {
        zone_id: String,
        record_id: String,
        name: String,
        record_type: RecordType,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    },

    DeleteRecord {
        zone_id: String,
        record_id: String,
    },

    UpsertRecord {
        zone_id: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    },

    // SRV Record Operations
    CreateSrvRecord {
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },

    UpsertSrvRecord {
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },

    // Preset Operations (from wizard)
    CreateDefaultXmppRecords {
        zone_id: String,
        domain: String,
        vm_ip: String,
    },

    VerifyDefaultXmppRecords {
        zone_id: String,
        domain: String,
        vm_ip: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SRV,
}

impl RecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordType::A => "A",
            RecordType::AAAA => "AAAA",
            RecordType::CNAME => "CNAME",
            RecordType::MX => "MX",
            RecordType::TXT => "TXT",
            RecordType::NS => "NS",
            RecordType::SRV => "SRV",
        }
    }
}
