//! NS actor events

#[derive(Debug, Clone)]
pub struct DnsProvider {
    pub name: String,
    pub provider_type: String,
}

#[derive(Debug, Clone)]
pub struct DnsDomain {
    pub domain: String,
    pub provider_name: String,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone)]
pub enum NsEvent {
    // Provider Events
    ProviderAdded {
        name: String,
        domains: Vec<(String, Vec<crate::calc::ns::DnsRecord>)>,
    },
    ProviderDeleted {
        name: String,
    },
    ProvidersListed {
        providers: Vec<DnsProvider>,
    },

    // Domain Events
    DomainAdded {
        provider_name: String,
        domain: String,
    },
    DomainDeleted {
        provider_name: String,
        domain: String,
    },
    DomainsListed {
        provider_name: String,
        domains: Vec<DnsDomain>,
    },

    // Record Events
    RecordAdded {
        provider_name: String,
        domain: String,
        record_id: String,
    },
    RecordDeleted {
        provider_name: String,
        domain: String,
        record_id: String,
    },
    RecordsListed {
        provider_name: String,
        domain: String,
        records: Vec<DnsRecord>,
    },

    // Progress & Errors
    Progress {
        operation: String,
        progress: f32,
        status: String,
    },
    Error {
        operation: String,
        error: String,
    },
}
