//! NS actor commands

#[derive(Debug, Clone)]
pub enum NsCommand {
    // Provider Management
    AddProvider {
        name: String,
        provider_type: String,
        api_token: String,
    },
    DeleteProvider { name: String },
    ListProviders,

    // Domain Management
    AddDomain { provider_name: String, domain: String },
    DeleteDomain { provider_name: String, domain: String },
    ListDomains { provider_name: String },

    // DNS Record Management
    AddRecord {
        provider_name: String,
        domain: String,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    },
    DeleteRecord {
        provider_name: String,
        domain: String,
        name: String,
        record_type: String,
    },
    ListRecords { provider_name: String, domain: String },

    // Refresh all DNS data
    RefreshAll,
}
