# Dure CLI Reference

Comprehensive reference for all `dure` CLI commands.

## Table of Contents

- [Overview](#overview)
- [Global Flags](#global-flags)
- [Commands](#commands)
  - [ns - Nameserver Management](#ns---nameserver-management)
  - [platform - Platform Management](#platform---platform-management)
  - [dns - DNS Lookup](#dns---dns-lookup)
  - [key - Key Management](#key---key-management)
  - [ssh - SSH Host Management](#ssh---ssh-host-management)
  - [audit - Audit Trail](#audit---audit-trail)
  - [crypt - Cryptographic Operations](#crypt---cryptographic-operations)
  - [site - Site Management](#site---site-management)
  - [info - Diagnostic Info](#info---diagnostic-info)
  - [init - Initialize Workspace](#init---initialize-workspace)
- [Exit Codes](#exit-codes)
- [Environment Variables](#environment-variables)

## Overview

Dure provides a command-line interface for headless operation. All commands operate on a **profile context** - operations are scoped to the current profile (default: "default").

```bash
dure <command> [subcommand] [options]
```

## Global Flags

These flags work with any command:

| Flag | Description |
|------|-------------|
| `-h, --help` | Print help information |
| `-V, --version` | Print version |

## Commands

### ns - Nameserver Management

Manage DNS nameservers and records across multiple providers (Cloudflare, GCP DNS, DuckDNS, Porkbun).

#### ns status

List all registered domains and their records.

```bash
dure ns status [domain]
```

**Arguments:**
- `[domain]` - Optional: show records for specific domain only

**Example:**
```bash
dure ns status
dure ns status example.com
```

#### ns add

Register a new domain with a DNS provider.

```bash
dure ns add <domain> --provider <provider> --token <token>
```

**Arguments:**
- `<domain>` - Domain name (e.g., www.example.com)
- `--provider` - DNS provider: cloudflare, gcloud, duckdns, porkbun
- `--token` - API token for the DNS provider

**Example:**
```bash
dure ns add example.com --provider cloudflare --token abc123...
```

#### ns del

Remove a domain from nameserver.

```bash
dure ns del <domain>
```

**Example:**
```bash
dure ns del example.com
```

#### ns insert

Add a DNS record to a domain.

```bash
dure ns insert <record_type> <domain> <value> [--apply]
```

**Arguments:**
- `<record_type>` - Record type: a, aaaa, txt, ns
- `<domain>` - Domain name
- `<value>` - Record value (IP for A/AAAA, text for TXT, nameserver for NS)
- `--apply` - Apply change to DNS provider immediately

**Example:**
```bash
dure ns insert a example.com 192.0.2.1 --apply
dure ns insert txt _acme-challenge.example.com "validation-token"
```

#### ns remove

Remove a DNS record from a domain.

```bash
dure ns remove <record_type> <domain> <value>
```

**Example:**
```bash
dure ns remove a example.com 192.0.2.1
```

---

### platform - Platform Management

Manage cloud platforms (GCP, Firebase, Supabase).

#### platform add

Add a new platform.

```bash
dure platform add <name> [--platform-type <type>]
```

**Arguments:**
- `<name>` - Platform name
- `--platform-type, -t` - Platform type: gcp, firebase, supabase (default: gcp)

**Example:**
```bash
dure platform add my-project --platform-type gcp
```

#### platform [external commands]

Platform-specific commands are dynamically routed based on platform type. See platform-specific documentation.

---

### dns - DNS Lookup

Perform DNS lookups with caching.

#### dns a

Query A records (IPv4 addresses).

```bash
dure dns a <domain>
```

**Example:**
```bash
dure dns a example.com
```

#### dns aaaa

Query AAAA records (IPv6 addresses).

```bash
dure dns aaaa <domain>
```

#### dns ns

Query NS records (nameservers).

```bash
dure dns ns <domain>
```

#### dns txt

Query TXT records.

```bash
dure dns txt <domain>
```

#### dns bastion

Add TXT record for bastion IP address (automatic bastion allow list).

```bash
dure dns bastion <ip>
```

**Example:**
```bash
dure dns bastion 203.0.113.42
```

---

### key - Key Management

Password manager with KeePass (.kdbx) support.

#### key status

List all keys in the current keyring.

```bash
dure key status
```

#### key add

Add a new key to the keyring.

```bash
dure key add <domain> <username> <password>
```

**Arguments:**
- `<domain>` - Domain/URL (e.g., www.dure.app)
- `<username>` - Username/email (e.g., admin@dure.co)
- `<password>` - Password/credential

**Example:**
```bash
dure key add api.example.com admin@example.com mypassword123
```

#### key del

Delete a key from the keyring.

```bash
dure key del <domain>
```

#### key save

Export keyring to KeePass file.

```bash
dure key save [output]
```

**Arguments:**
- `[output]` - Output file path (default: ./exported_keys.kdbx)

**Example:**
```bash
dure key save ~/backup/keys.kdbx
```

#### key load

Import keyring from KeePass file (replaces current keyring).

```bash
dure key load <input>
```

**Arguments:**
- `<input>` - Input KeePass file path (.kdbx)

**Example:**
```bash
dure key load ~/backup/keys.kdbx
```

---

### ssh - SSH Host Management

Manage SSH hosts and perform automated server initialization.

#### ssh status

Show list and status of SSH hosts.

```bash
dure ssh status
```

#### ssh add

Add SSH host to configuration.

```bash
dure ssh add <host> [--pass <password>] [--prvkey <path>] [--port <port>]
```

**Arguments:**
- `<host>` - SSH connection string (username@hostname)
- `--pass` - SSH password (optional)
- `--prvkey` - Path to private key file (optional)
- `--port` - SSH port (default: 22)

**Example:**
```bash
dure ssh add root@192.0.2.1 --prvkey ~/.ssh/id_ed25519
dure ssh add admin@server.example.com --pass mypassword --port 2222
```

#### ssh del

Remove SSH host from configuration.

```bash
dure ssh del <host>
```

#### ssh init

Initialize SSH host (automated server setup).

Installs:
- Swap space
- nftables firewall
- Dure server components

```bash
dure ssh init <host>
```

**Example:**
```bash
dure ssh init root@192.0.2.1
```

---

### audit - Audit Trail

Manage audit logs (action history).

#### audit status

Review action history (most recent 50 records).

```bash
dure audit status
```

#### audit clear

Wipe all audit records (requires confirmation).

```bash
dure audit clear
```

---

### crypt - Cryptographic Operations

Encryption/decryption using Ed25519 key pairs.

#### crypt status

Show base public key for the system.

```bash
dure crypt status
```

#### crypt enc

Encrypt data for a recipient.

```bash
dure crypt enc <recipient_pubkey> <data> [--hex]
```

**Arguments:**
- `<recipient_pubkey>` - Recipient's public key (base64 or hex)
- `<data>` - Data to encrypt
- `--hex` - Output as hex instead of base64

**Example:**
```bash
dure crypt enc "ABC123...=" "secret message"
```

#### crypt dec

Decrypt data.

```bash
dure crypt dec <encrypted_data> [--raw]
```

**Arguments:**
- `<encrypted_data>` - Encrypted data (base64 or hex)
- `--raw` - Output raw bytes instead of UTF-8 text

**Example:**
```bash
dure crypt dec "XYZ789..."
```

---

### site - Site Management

Manage sites for site-to-site communication.

#### site status

List all configured sites.

```bash
dure site status
```

#### site add

Add a new site for site-to-site communication.

```bash
dure site add <domain> --public-key <key>
```

**Arguments:**
- `<domain>` - Domain name (e.g., example.com)
- `--public-key` - Public key for authentication

**Example:**
```bash
dure site add partner.example.com --public-key "ABC123..."
```

#### site del

Delete a site.

```bash
dure site del <domain>
```

---

### info - Diagnostic Info

Show diagnostic metadata about the workspace.

```bash
dure info
```

Displays:
- Profile information
- Database location
- Configuration paths
- System information

---

### init - Initialize Workspace

Initialize a new workspace with database and configuration.

```bash
dure init [--prefix <prefix>] [--force]
```

**Arguments:**
- `--prefix` - Issue ID prefix (e.g., "bd")
- `--force` - Overwrite existing database

**Example:**
```bash
dure init --prefix dure
dure init --force  # Reinitialize (dangerous!)
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Network error |
| 4 | Authentication error |
| 5 | Not found |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Logging level (error, warn, info, debug, trace) |
| `DURE_PROFILE` | Override default profile |
| `DURE_DB_PATH` | Override database path |

**Example:**
```bash
RUST_LOG=debug dure ns status
DURE_PROFILE=production dure platform add my-project
```

## Notes

- All commands operate on profile-scoped data (default profile: "default")
- Database operations use Diesel with SQLite (or PostgreSQL for production)
- DNS operations cache results for performance
- SSH operations use russh (pure Rust SSH implementation)
- All cryptographic operations use Ed25519/X25519
- Keyring storage is platform-specific (keychain/secret-service/DPAPI)

## See Also

- **[ARCH.sudolang.md](./ARCH.sudolang.md)** - Architecture details
- **[../CLAUDE.md](../CLAUDE.md)** - Complete project guide
- **[TROUBLESHOOTING.md](./TROUBLESHOOTING.md)** - Common issues
