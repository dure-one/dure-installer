# Dure - Community App

## Why This Project Exists
Its attempt to providing distributed e-commerce experience for small shop owner.

---

## TL;DR

### The Problem
Today's e-commerce solutions are server-centric, which makes them attractive targets for hackers and increases security risk. A distributed store
 architecture addresses this vulnerability — though building such a service comes with its own challenges.

- **Hosting Store Front with Payment Gateway** require DNS Settings, Wasm Static Hosting, Web-DB like supabase, firebase supports
- **Managing Shared Stores** provide store managing features. Products, Orders, Shipments, Accounts, Dure(Shared) Managements.
- **Ecommerce Client** managing multiple store front client.

### The Solution
**dure** is a distributed ecommerce client/hosing with a single tool.

- **Identity Mgmt** : Private/Public Key for personal identity, Firebase/Supabase identity. Attestation WASM/EGUI Apps with github sigstore.
- **Guest Front with WASM** : Providing minimum Guest identity for customer. Listing Reviews. Messages. Stores. Carts.
- **Store Front with WASM (only for Service Providers)** : Listing Promotions, Products, Payments.
- **Hosting Mgmt with EGUI** : DNS (octodns), DB (Firebase, Supabase), Site (FB/SB CloudFunctions)
- **Store Mgmt with EGUI** : Managing Promotions, Products, Orders, Shipments, Accountings, Dure (Shared Listing, Shipments with other store)
- **Database** : Diesel ORM with embedded migrations (SQLite for all platforms, optional PostgreSQL for desktop)
- All operations are ready for modification by LLM.
- Diesel uses SQLite3MultipleCiphers backend for desktop, sqlite3 for diesel, postgresql for firebase or supabase.

## 📚 Documentation

**For developers and AI assistants:**
- **[CLAUDE.md](./CLAUDE.md)** - Complete project guide (start here!)
- **[docs/PROJECT_SUMMARY.md](./docs/PROJECT_SUMMARY.md)** - Architecture overview and quick reference
- **[docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)** - Commands, patterns, and common tasks
- **[docs/INDEX.md](./docs/INDEX.md)** - Complete documentation index

**Note**: Some documentation files reference a different project and are being updated. See docs/INDEX.md for current status.

## 🏗️ Architecture

**Dure uses MVVM (Model-View-ViewModel) architecture with actor-based concurrency:**

- **View**: egui UI (Desktop/Android), WASM (Browser), CLI (Headless)
- **ViewModel**: Async actor coordinator with unified API for all platforms
- **Model**: Business logic (calc layer), Database (Diesel ORM), External APIs

**Key Benefits:**
- ✅ Clean separation between UI and business logic
- ✅ Async by default - all I/O is non-blocking  
- ✅ Cross-platform - same ViewModel for Desktop, CLI, Android, WASM
- ✅ Testable - actors can be tested independently
- ✅ Progress reporting - built-in progress events for long operations
- ✅ ~60% code reduction - event-driven vs blocking calls

**MVVM Implementation Status:**
- ✅ Core infrastructure complete (16/16 tasks)
- ✅ UI migration in progress (6 operations migrated)
- 📚 [Migration Guide](./docs/MVVM_MIGRATION_GUIDE.md) - Pattern and examples
- 📊 [Migration Status](./docs/MVVM_MIGRATION_STATUS.md) - Detailed progress
- 🚧 [Blockers](./docs/MVVM_BLOCKERS.md) - Challenges and solutions

## Why Dure?

### General Comparison by Services
| Platform | Hosting | Payment Options | Best Use Cases |
|----------|---------|-----------------|----------------|
| **Dure** | GCP, Firebase, Supabase, Cafe24 | Portone, KakaoPay | - |
| **Shopify** | Fully hosted (managed) | Shopify Payments, PayPal, Stripe, 100+ gateways | All-around solution, scalability, quick launch, dropshipping, mobile commerce |
| **Wix** | Fully hosted (managed) | Wix Payments, PayPal, Stripe (limited options) | Beginners, small catalogs, artists/creatives, side businesses |
| **Magento (Adobe Commerce)** | Self-hosted or Cloud | PayPal, Braintree, Authorize.net, custom integrations | Enterprise, large catalogs (100K+ products), complex B2B, multi-brand |

### Features
(Progress : ⬜ Plan &nbsp;|&nbsp; ✏️ Ing &nbsp;|&nbsp; 🔍 Test &nbsp;|&nbsp; ✅ Done)
| Section | Type | DB | CLI | GUI | WASM |
|---------|-------|------|-----|-----|------|
| **Client Operations** | | | | | |
| Audit | Client | Db | 🔍 Test | ⬜ Plan | ⬜ Plan |
| DNS Client | Client | Hashmap | 🔍 Test | ⬜ Plan | ⬜ Plan |
| CRYPT | Client | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| KEY Mgmt(Passkey, FIDO2) | Client | Keystore | ✏️ Ing | ⬜ Plan | ⬜ Plan |
| Sites | Client | Keystore | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| **Hosting Operations** | | | | | |
| Platform | Hosting | Config File | ⬜ Plan | ⬜ Plan | - |
| SSH | Hosting | Config File | ⬜ Plan | ⬜ Plan | - |
| DNS Cloud | Hosting | Config File | ⬜ Plan | ⬜ Plan | - |
| **SSH/Server Operations** | | | | | |
| ACME(lego) | SSH | DB Secured | 🔍 Test | ⬜ Plan | - |
| NFTABLES | SSH | DB Secured | 🔍 Test | ⬜ Plan | - |
| WSS(TUNGSTENITE) | SSH | DB Secured | 🔍 Test | ⬜ Plan | - |
| Hosting | SSH | DB Secured | ✏️ Ing | ⬜ Plan | - |
| **SITE Operations** | | | | | |
| Auth | Site | Keystore | ✏️ Ing | ⬜ Plan | ⬜ Plan |
| Directory(Member) | Site | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Role | Member | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Product | Site | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Order | Site | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Payment | Order | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Review | Product | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Device | Member | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Channel | Site | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Messages | Channel | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Event | Channel | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Webhook | Channel | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Reaction | Messages | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Direct Messages | Member | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Threads | Channel | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |
| Poll | Channel | DB | ⬜ Plan | ⬜ Plan | ⬜ Plan |

### Usage

- Clients Type : CLI, GUI, WASM, MCP
- Server Type : GCP(Fully Automatic), Cafe24(SSH only), Firebase(RealtimeDB), Supabase

#### Start Service
```bash
# Create configuration file

# Run configuration Check
dure hosting check

# Run Initiating Process
dure hosting init
```

#### Start TRAY/GUI Guest Client (Desktop/Android/WASM)
```bash
# client tray mode
dure --tray 

# client gui mode without tray
dure --gui 
```
* Dure eGui Android App with [Android App Functions](https://developer.android.com/ai/appfunctions)

---

## Design Philosophy

### 1. Agent-First Design

Every command supports `--json` for AI coding agents:

```bash
dure list --json | jq '.issues[] | select(.priority <= 1)'
dure ready --json  # Structured output for agents
dure show bd-abc123 --json
```

For routine operator or agent use, prefer `RUST_LOG=error br ...` to suppress internal Rust dependency logs while preserving normal stdout/JSON output:

```bash
RUST_LOG=error dure ready --json
RUST_LOG=error dure sync --flush-only
```

### 2. Separate egui comonents with data preparation. and put some cache in between.

### 3. All api service and client is generate by ASYNCAPI generator. strictly controlled.

---

## Quick Start

### 1. Setup Hosting/Domain/Site
Create Hosting Server and Ready for Service.
1. Select DNS Registar and get API key(Porkbun, Cloudflare) or Domain Name(DuckDns).
2. Select NS Server and API key(Duckdns, Porkbun, Cloudflare).
3. Select VM Cloud(GCE or Cafe24) and get API Key. Prepare SSH Account and Key or Password.
4. Initial Remote Host Setup for Dure Websocket Server.

### 2. Setup Dure
Set Configuration to Dure WebServer.
1. Start to connect other dure server.
2. Browsing products from other server.
3. Order from other server.

### 3. Setup Store
Set Configuration to Store WebServer.
1. Start to connect Own dure server.
2. Create Products to server.
3. Start to sell.

---

## Commands

### Object Hierachy Tree
```
Hosting -- Site   -- Member -- Role 
(server) |         |         |- Device
(desktop)|         |         |- Direct Message
         |         |- Channel -- Message -- Reaction
         |         |          |- Thread
         |         |          |- Poll
         |         |          |- Event
         |         |          |- Webhook
         |         |- Product - Review
         |         |- Order - Payment
         |         |- Auth
         |- DNS(DOH) Client/Cloud API
         |- ACME(SSL Registration)
         |- IPTABLES(SSH Port Control)
         |- TUNGSTENITE(WSS/HTTPS)
         |- VM/SSH CLIENT(ssh2-rs)
         |- Audit

CLIENT -- TUNGSTENITE(WSS/HTTPS)
(desktop)|- ATTESTATION(sigstore)
(android)|- ENCDEC(X25519+ChaCha20)
(+wasm)  |- KEYMGMT(LOGIN,SSL,MSG)
         |- Audit
         |-------(+wasm)-----------------------------
         |- Site   -- Member -- Role 
                    |         |- Device
                    |         |- Direct Message
                    |- Channel -- Message -- Reaction
                    |           |- Thread
                    |           |- Poll
                    |           |- Event
                    |           |- Webhook
                    |- Product - Review
                    |- Order - Payment
                    |- Auth
```

### Audit Operations(Client)
| Command | Description | Example |
|---------|-------------|---------|
| `audit status` | Review action history | `dure audit status` |
| `audit clear` | Wipe logs | `dure audit clear` |

### DNS Client Operations(Client)
| Command | Description | Example |
|---------|-------------|---------|
| `dns a` | Get A dns records of example.com | `dure dns a example.com` |
| `dns aaaa` | Get AAAA dns records of example.com | `dure dns aaaa example.com` |
| `dns sshfp` | Get SSHFP dns records of example.com | `dure dns a example.com` |
| `dns txt` | Get TXT dns records of example.com | `dure dns txt example.com` |
| `dns bastion` | Add TXT record of bastion ip address of ssh allow ip | `dure dns bastion 29.392.182.22` |

```
DOH + DNSSEC + A, AAAA, TXT(Server Pubkey/Primary Pubkey) Resolving
```

### CRYPT Operations(Client)
Encryptiong Decryption Data(ChaCha20-Poly1305)
| Command | Description | Example |
|---------|-------------|---------|
| `crypt enc` | Encrypt data to send  | `dure crypt enc ${recepient pubkey} ${data}` |
| `crypt dec` | Decrypt data to receive | `dure crypt dec ${sender_pubkey} ${data}` |

```
* refer docs/MSG_EXCHANGE.md section (2. X25519 + ChaCha20-Poly1305 (Industry Standard))
```

### KEYMGMT Operations(Client)
Local Key Management and Exchange(Keepass)
| Command | Description | Example |
|---------|-------------|---------|
| `key save` | save keychain to keypass format | `dure save export xxx.kdb` |
| `key load` | load keychain from keypass format | `dure load import xxx.kdb` |
| `key status` | list all keys | `dure key status` |
| `key add` | Add key to current keychain | `dure key add www.dure.app nikescar@gmail.com password` |
| `key del` | Delete key from current keychain | `dure key del www.dure.app` |

```

# passkey client authentication for egui and wasm.

ES256 (ECDSA using P-256 curve with SHA-256)
1. server send request for passkey
2. browser or client generate public/private keypair
3. private key is stored in storage with encryption
4. send public key to server
example scenario is in go-webauthn/examples/passkey_login.rs

# fido2 authentication for egui and wasm.

ES256 (ECDSA P-256 with SHA-256)
1. server send challenge string to browser or client
2. your device hardware generate a unique public/private keypair
3. private key is stored in storage with encryption
4. send public key to server

# chacha20 authentication for site2site.

1. get counter domain secret key(ES256 (ECDSA P-256 with SHA-256)) txt record from dnsclient
2. extract pubkey from txt record
3. encrypt chacha secret key with the pubkey and send them to counter part(https://example.com/api/ws) and encrypt auth message with chacha secret key.
4. receive results and decrypt with chacha secret key.
https://pycryptodome.readthedocs.io/en/v3.10.4/src/cipher/chacha20_poly1305.html

# CASE1. Login from Device(CLI/GUI) to Ownsite(Deive -> Ownsite)
 - Connection : websocket auth. passkey or fido2.
# CASE2. Login from Device(CLI/GUI) to Otherssite(Device -> Otherssite)
 - Connection : websocket auth. passkey or fido2.
 - Site2Site : websocket auth. site passkey.
# CASE3. Login from Browser(WASM) to Ownsite(Browser -> Ownsite)
 - Login : websocket auth. passkey or fido2.
# CASE4. Login from Browser(WASM) to Otherssite(Browser -> Otherssite)
 - Login : websocket auth. passkey or fido2.
 - Site2Site : websocket auth. site passkey.

# Websocket Auth Endpoints(Passkey/Fido2)
# Register/Login/Logout

# Site2Site Auth Endpoints(Passkey/)
# Register/Login/Logout
```

### Dure - Sites Operations(Client)
| Command | Description | Example |
|---------|-------------|---------|
| `site list` | List site on client  | `dure site list "www.dure.com"` |
| `site add` | Add site to client | `dure site add "www.dure.com"` |
| `site del` | Remove site to client | `dure site del "www.dure.com"` |

```
* if site address is on ssh host, it is trying admin. auth is keystore.
```

### Platform Operations(GCP, Firebase, Supabase)
| Command | Description | Example |
|---------|-------------|---------|
| `platform status` | List platform on client  | `dure platform status` |
| `platform add` | Add platform to client | `dure platform add "name"` |
| `platform del` | Remove platform to client | `dure platform del "name"` |
| `platform init` | Run platform initiation | `dure platform init "name"` |

### DNS Cloud Operations(Hosting)
| Command | Description | Example |
|---------|-------------|---------|
| `ns status` | List all registered domain and its record to ns | `dure ns status` |
| `ns status www.example.com` | List records for the domain | `dure ns status www.example.com` |
| `ns add` | Add domain to ns | `dure ns add www.example.com` |
| `ns del` | Delete domain to ns | `dure ns del www.example.com` |
| `ns insert` | Insert record(A,TXT) to domain | `dure ns insert a www.example.com 111.111.111.111` |
| `ns remove` | Remove record(A,TXT) to domain | `dure ns remove a www.example.com 111.111.111.111` |

```
Cloudflare, CloudDNS, DuckDNS, Porkbun

* Examples for ns commands to set A,TXT records:
https://github.com/octodns/octodns-cloudflare
https://github.com/octodns/octodns-ns1
https://github.com/octodns/octodns-googlecloud
https://github.com/major/octodns-porkbun
Duckdns
https://github.com/libdns/duckdns/blob/master/client.go
General
https://github.com/qdm12/ddns-updater

* TXT Record Struct
durepubkey=my_system_chacha_public_key ; # pubkey
```

### SSH Operations(Hosting)
| Command | Description | Example |
|---------|-------------|---------|
| `ssh status` | Show list and status of ssh hosts | `dure ssh status` |
| `ssh add` | Add host to list | `dure ssh addhost username@dure.com --pass password --prvkey ~/.ssh/id_ed25519` |
| `ssh del` | Delete host from list | `dure ssh delhost username@dure.com` |

### ACME Operations(SSH)
Cloudflare, CloudDNS, DuckDNS, Porkbun
| Command | Description | Example |
|---------|-------------|---------|
| `acme status` | Get certificate list on system | `dure acme status` |
| `acme install` | Install acme.sh to system | `dure acme install` |
| `acme issue` | Issue standard certificate for example.com | `dure acme issue example.com` |
| `acme renew` | Renew certificate(every 60 days) | `dure acme renew example.com` |

```
* acme command to use
https://github.com/acmesh-official/acme.sh
https://github.com/acmesh-official/acme.sh/wiki/How-to-install
issue: acme.sh --issue --standalone -d example.com -d www.example.com -d cp.example.com
renew: acme.sh --renew -d example.com --force
```

### NFTABLES Operations(SSH)
| Command | Description | Example |
|---------|-------------|---------|
| `nft status` | List nftables ruleset | `dure nft show` |
| `nft whitelist` | Add nftables whitelist ip for ssh connection | `dure nft whitelite 111.111.111.111` |
| `nft remove` | Remove nftables whitelist ip for ssh connection | `dure nft remove 111.111.111.111` |
| `nft update` | Update nftables bastion ip | `dure nft show` |

```
* only open 80, 443 ports to world.
* whitelist add means sshport 22 to specific ip. https://www.softworx.at/en/nftables-cheat-sheet-useful-commands-for-nft-part-1/
```

### WSS(TUNGSTENITE) Operations(SSH)
Session Creation + Management
| Command | Description | Example |
|---------|-------------|---------|
| `wss status` | Get status of https/websocket server | `dure wss status` |
| `wss start` | Start https/websocket server | `dure wss start` |
| `wss stop` | Stop https/websocket server | `dure wss stop` |
| `wss update` | Update https/websocket server admin auth | `dure wss update` |

### Hosting Operations(SSH)
* this operation is combining multiple tasks.

| Command | Description | Example |
|---------|-------------|---------|
| `hosting check` | Required configuration and validation check | `dure hosting check` |
| `hosting init` | Initialize hosting | `dure hosting init` |
| `hosting show` | Show hosting details and configurations | `dure hosting show www.asset.com` or `dure hosting show` |
| `hosting select` | Select hosting host for ops | `dure hosting select www.asset.com` |
| `hosting deselect` | Deselect hosting host for ops | `dure hosting deselect www.asset.com` |
| `hosting close` | Close hosting | `dure hosting close www.asset.com` |
| `hosting reopen` | Reopen closed | `dure hosting reopen www.asset.com` |
| `hosting delete` | Delete | `dure hosting delete www.asset.com` |
| `hosting list` | Show bot's server memberships | `dure hosting list` |

```
1. DNS API
https://github.com/acmesh-official/acme.sh/tree/master/dnsapi
https://github.com/octodns/octodns/blob/main/CHANGELOG.md#v0915---2022-02-07---where-have-all-the-providers-gone
https://github.com/octodns/octodns-cloudflare/
https://github.com/octodns/octodns-googlecloud/
https://github.com/octodns/octodns-route53/
https://github.com/googleapis/google-cloud-rust/blob/main/tests/dns/src/lib.rs
2. GCP API
https://github.com/OutlineFoundation/outline-apps/blob/master/server_manager/cloud/gcp_api.ts
https://github.com/googleapis/google-cloud-rust/blob/main/guide/samples/src/compute/compute_instances_create.rs
https://github.com/Byron/google-apis-rs/tree/main/gen/compute1/src
3. CAFE24 VPS(UBUNTU) SSH API
https://github.com/dirien/quick-bites/blob/main/rust-ssh/src/main.rs
```

#### Init Operations
1. check proper application-wide db file is created in config dir and open it.
2. create device identity with machineid-rs crate(reference/machineid-rs/README.md) and save them in sqlite storage.
3. create per device privatekey and publickey for identity ### 2. X25519 + ChaCha20-Poly1305 (Industry Standard) in docs/MSG_EXCHANGE.md and save them in sqlite storage.

### Dure - Auth Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `auth create` | Create a login info | `dure auth create ` |
| `auth status` | Status login/logout multiple sites | `dure auth status` |
| `auth login` | Login | `dure auth delete "Server" 9876543210` |
| `auth logout` | Logout | `dure auth modify "Server" 9876543210 "Product Name" "Product Category" "Image" "Contents"` |

### Dure - Directory(Member) Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `member list` | List members in a server | `dure member list "Host1"` |
| `member list --limit` | List members with custom limit | `dure member list "Host1" --limit 100` |
| `member info` | Get member details (roles, join date, etc.) | `dure member info "Host1" "@alice"` |
| `member info` (by ID) | Get member details by user ID | `dure member info "Host1" 1234567890` |
| `member kick` | Kick a member (confirmation required) | `dure member kick "Host1" "@spammer" --reason "Spam"` |
| `member ban` | Ban a member permanently | `dure member ban "Host1" "@troll" --reason "Repeated violations"` |
| `member unban` | Unban a member | `dure member unban "Host1" "@reformed"` |
| `member timeout` | Timeout a member (e.g., 1 hour = 3600s) | `dure -y member timeout "Host1" "@spammer" 3600 --reason "Spam"` |
| `member timeout` (remove) | Remove a timeout (set duration to 0) | `dure -y member timeout "Host1" "@spammer" 0` |

```
* there are 2 types of members. 1.owner member for owner devices. 2. store member for other stores.
* owner member id does not allow dot(.) in their id.
* store memeber id does allow dot(.) in  their id.
* if store member got message, it will sync with the store wss.
```

### Dure - Role Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `role list` | List roles in a server | `dure role list "Host1"` |
| `role create` | Create a role | `dure role create "Host1" "Contributor" --color "00ff00"` |
| `role assign` | Assign a role to a member | `dure role assign "Host1" "@alice" "Contributor"` |
| `role remove` | Remove a role from a member | `dure role remove "Host1" "@alice" "Contributor"` |
| `role edit` | Edit a role (name, color, hoist, mentionable) | `dure role edit "Host1" "Contributor" --name "Core Contributor" --color 00ff00 --hoist --mentionable` |
| `role delete` | Delete a role (confirmation required) | `dure role delete "Host1" "OldRole"` |

#### Detailed Services by Role
| Role | A.Domain Mgmt | B.DNS Mgmt | C.Web Mgmt | D.DB Mgmt | E.MSG MGMT | F.MSG Receiving |
| ---- | ------------- | ---------- | ---------- | --------- | ----- | ----- |
| Guest | O | O | O | - | - | O |
| Service Provider(Store Owner) | O | O | O | O | O | O |

#### Data Managed by Role
| Role | Guest | Service Provider | 
|-----|-----|-----|
| **A.Dure** | O | O |
| **A1.OtherStores** | O | O |
| **A2.Groups** | O | O |
| **A3.Reviews** | O | O |
| **A4.Messages** | O | O |
| **B.Store** | | O |
| **B1.Promotions** | | O |
| **B2.Products** | | O |
| **B3.Orders** | | O |
| **B4.Accountings** | | O |

### Dure - Product Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `product create` | Create a product from a message | `dure product create "Server" "Product Name" "Product Category" "Image" "Contents"` |
| `product list` | List active products in a channel | `dure product list "Server"` |
| `product delete` | Delete a product | `dure product delete "Server" 9876543210` |
| `product modify` | Modify a product | `dure product modify "Server" 9876543210 "Product Name" "Product Category" "Image" "Contents"` |

### Dure - Order Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `order create` | Create a order from known store | `dure order create "Server" "Product Ids" "Counts"` |
| `order list` | List active orders in a channel | `dure order list "Server"` |

### Dure - Payment Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `payment create` | Create a payment from a message | `dure payment create "Server" 1234567890123456 "Payment Method"` |
| `payment list` | List active payments in a channel | `dure payment list "Server"` |
| `payment verify` | Verify payment from Payment Gateway | `dure payment verify "Server" 123456788192 ${data}"` |

```
* when receive payment confirmation from pg server, payment verify initiated.
```

### Dure - Review Operations(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `review create` | Post review to Server | `dure review create "Server" Order_ID "Review Contents"` |
| `review list` | Post to review | `dure review list "Server"` |

### Dure - Device Operations(Member)
| Command | Description | Example |
|---------|-------------|---------|
| `device list` | Show devices in all hostings | `dure device list "Host1" "Member1"` |
| `device info` | Get device metadata | `dure device info "Host1" "Member1" "Device1"` |

### Dure - Channel Management(SITE)
| Command | Description | Example |
|---------|-------------|---------|
| `channel list` | Display all channels across all servers | `dure channel list` |
| `channel list --server "Host1"` | Channels in a specific server | `dure channel list --server "Host1"` |
| `channel info` | Retrieve channel details | `dure channel info "#channel"` |
| `channel create` | Add text/voice/forum channels | `dure channel create "Server" "name" --type voice` |
| `channel forum-post` | Post forum threads | `dure channel forum-post "#feedback" "title" "content"` |
| `channel edit` | Update channel settings | `dure channel edit "#channel" --topic "new topic" --slowmode 10` |
| `channel set-permissions` | Control access permissions | `dure channel set-permissions "#channel" "Role" --allow send_messages` |
| `channel delete` | Remove channels | `dure channel delete "#channel"` |

### Dure - Messaging Operations(CHANNEL)
Send a message to any channel your bot has access to:
| Command | Description | Example |
|---------|-------------|---------|
| `message send` | Deliver messages to channels | `dure message send "#general" "Hello from the terminal!"` |
| `message list` | Retrieve recent messages (default: 10) | `dure message list "#channel"` |
| `message search` | Find messages by content or author | `dure message search "#channel" "query"` |
| `message get` | Fetch specific message by ID | `dure message get "#channel" ID` |
| `message history` | Export historical records | `dure message history "#channel" --days 7` |
| `message reply` | Reply to specific messages | `dure message reply "#channel" ID "response"` |
| `message edit` | Modify bot's own messages | `dure message edit "#channel" ID "new text"` |
| `message delete` | Remove messages (confirmation required) | `dure message delete "#channel" ID` |
| `message bulk-delete` | Remove multiple messages at once | `dure message bulk-delete "#channel" ID1 ID2 ID3` |

### Dure - Event Operations(CHANNEL)
| Command | Description | Example |
|---------|-------------|---------|
| `event list` | List scheduled events  | `event list` |

### Dure - Webhook Operations(CHANNEL)
| Command | Description | Example |
|---------|-------------|---------|
| `webhook list` | List webhooks  | `webhook list` |

### Dure - Reaction Management(MESSAGE)
| Command | Description | Example |
|---------|-------------|---------|
| `reaction add` | Add emoji reactions | `dure reaction add "#channel" MESSAGE_ID "👍"` |
| `reaction remove` | Remove reactions | `dure reaction remove "#channel" MESSAGE_ID "👍"` |
| `reaction list` | View all reactions on a message | `dure reaction list "#channel" MESSAGE_ID` |
| `reaction users` | See who reacted with specific emoji | `dure reaction users "#channel" MESSAGE_ID "👍"` |

### Dure - Direct Messages(MEMBER)
| Command | Description | Example |
|---------|-------------|---------|
| `dm send` | Send private messages | `dure dm send "@user" "message"` |
| `dm list` | View conversation history | `dure dm list "@user"` |

### Dure - Threads(CHANNEL)
| Command | Description | Example |
|---------|-------------|---------|
| `thread create` | Create a thread from a message | `dure thread create "#general" 1234567890123456 "Discussion Thread"` |
| `thread list` | List active threads in a channel | `dure thread list "#general"` |
| `thread send` | Send a message to a thread | `dure thread send 9876543210 "Replying in the thread"` |
| `thread send --file` | Send with file attachment | `dure thread send 9876543210 "Here's the file" --file ./data.json` |
| `thread archive` | Archive a thread | `dure thread archive 9876543210` |
| `thread unarchive` | Unarchive a thread | `dure thread unarchive 9876543210` |
| `thread rename` | Rename a thread | `dure thread rename 9876543210 "New Thread Name"` |
| `thread add-member` | Add a member to a thread | `dure thread add-member 9876543210 "@alice"` |
| `thread remove-member` | Remove a member from a thread | `dure thread remove-member 9876543210 "@alice"` |

### Dure - Poll Operations(CHANNEL)
| Command | Description | Example |
|---------|-------------|---------|
| `poll create` | Create a poll (via serve mode action) | See the Serve Mode guide for poll_send details |
| `poll results` | View poll results | `dure poll results "#general" 1234567890123456` |
| `poll end` | End a poll early | `dure poll end "#general" 1234567890123456` |

### Global
| Command | Description | Example |
|---------|-------------|---------|
| (server mode) | | |
| `--serv` | Run server mode | `dure --serv` |
| (client mode) | | |
| `--tray` | Run gui mode(default)  | `dure --tray` |
| (commandline mode) | | |
| `--json` | Structured output format | `dure --json message list "#channel"` |
| `--token` | Override bot token | `dure --token YOUR_TOKEN message send "#channel" "text"` |
| `-y` / `--yes` | Skip confirmations | `dure -y message delete "#channel" ID` |
| `--profile` | Switch permission levels | `dure --profile readonly channel list` |

## Configuration

dure uses layered configuration:

1. **CLI flags** (highest priority)
2. **Environment variables**
3. **Project config**: `.dure/config.yaml`
4. **User config**: `~/.config/dure/config.yaml`
5. **Defaults** (lowest priority)

### Example Config

[embedmd]:# (mobile/config.example.yml)
```yml
# Dure Application Configuration
# platform | domain | server

# Platform identification
platform:
  name: "mylaptop"

# Domain and SSL certificate configuration
domain:
  # Primary domain name (will register both example.com and *.example.com)
  name: "example.com"
  
  # DNS provider: "cloudflare", "duckdns", "gcloud", or "porkbun"
  dns_provider: "cloudflare"
  
  # SSL certificate configuration
  cert:
    # Email for Let's Encrypt notifications
    email: "admin@example.com"
    # Certificate paths (auto-updated by 'dure acme issue')
    cert_path: ""
    key_path: ""
    issuer_path: ""
  
  # Cloudflare DNS credentials (if dns_provider = "cloudflare")
  cloudflare:
    # Option 1: Use API Token (recommended)
    api_token: "your_cloudflare_api_token_here"
    # Option 2: Use Email + API Key (legacy)
    # email: "you@example.com"
    # api_key: "your_cloudflare_api_key_here"
  
  # DuckDNS credentials (if dns_provider = "duckdns")
  duckdns:
    token: ""
  
  # Google Cloud DNS credentials (if dns_provider = "gcloud")
  # Option 1: Using service account file only
  #   GCE_PROJECT + GCE_SERVICE_ACCOUNT_FILE
  # Option 2: Using default credentials with impersonation
  #   GCE_PROJECT + GCE_IMPERSONATE_SERVICE_ACCOUNT
  # Option 3: Using service account file with impersonation
  #   GCE_PROJECT + GCE_SERVICE_ACCOUNT_FILE + GCE_IMPERSONATE_SERVICE_ACCOUNT
  gcloud:
    project: ""  # Required for all options
    service_account_file: ""  # Optional: path to service account JSON file
    impersonate_service_account: ""  # Optional: target service account email to impersonate
  
  # Porkbun DNS credentials (if dns_provider = "porkbun")
  porkbun:
    api_key: ""
    secret_api_key: ""

# Server hosting configuration
server:
  # Web hosting: "GCE", "VPS", "CLOUDFLARE_PAGES", or "FIREBASE_HOSTING"
  web_provider: "GCE"
  web_provider_token: ""
  # Database: "GCE", "GCP_CLOUDSQL", or "SUPABASE"
  db_provider: "GCE"

```

### Config Commands

```bash
# Show all config
dure config list

# Get specific value
dure config get id.prefix

# Set value
dure config set defaults.priority=1

# Open in editor
dure config edit
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `DURE_DB` / `DURE_DATABASE` | Override database path |
| `DURE_JSONL` | Override JSONL path (requires `--allow-external-jsonl`) |
| `RUST_LOG` | Logging level (debug, info, warn, error) |

Recommended default for normal CLI use:

```bash
export RUST_LOG=error
```

This keeps successful commands readable by suppressing low-level dependency logging. Remove or override it when debugging `dure` internals.

---

## AI Agent Integration

dure is designed for AI coding agents. See [AGENTS.md](AGENTS.md) for:

- JSON output schemas
- Workflow patterns
- Integration with MCP Agent Mail
- Robot mode flags
- Best practices

You can also emit machine-readable JSON Schema documents directly:

```bash
dure schema all --format json | jq '.schemas.Issue'
dure schema issue-details --format toon
```

---

## License
Dual-licensed under MIT OR Apache-2.0. See LICENSE-MIT and LICENSE-Apache-2.0.

---

## Star History

[![Star History Chart](https://api.star-history.com/image?repos=nikescar/dure&type=date&legend=top-left)](https://www.star-history.com/)

--- 

<details markdown>
<summary> Todos </summary>

## Todos


---
</details>
