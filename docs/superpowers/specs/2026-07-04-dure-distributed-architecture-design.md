# Dure Distributed E-Commerce Architecture Design

**Date:** 2026-07-04  
**Status:** Approved  
**Author:** Claude Sonnet 4.5 (with user guidance)

## Executive Summary

This document specifies the distributed e-commerce architecture for Dure, a federated platform enabling small shop owners to partner and share product catalogs while maintaining independent operations. The system uses a Chief-mediated registry for group membership, DNS TXT records for site-to-site authentication, and OAuth for guest authentication. Each shop operates as a single-instance deployment (Debian VM + SQLite) optimized for simplicity.

## Design Goals

1. **Simplicity First** - Single-instance deployment, SQLite backend, no horizontal scaling complexity
2. **Federation Without Central Control** - Shops maintain autonomy while benefiting from group product discovery
3. **Privacy by Design** - Guest data and orders are never shared between shops
4. **Decentralized Trust** - Public key cryptography via DNS TXT records, no shared secrets
5. **Scalability Limit** - Target small shops (100s-1000s of products); larger shops graduate to enterprise platforms

---

## Layer 1: Infrastructure Architecture

### Standard Deployment

Each Dure shop runs as a single-instance deployment:

```
┌─────────────────────────────────────────────────┐
│ Google Cloud Platform (GCP)                     │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │ Debian VM Instance                         │ │
│  │                                            │ │
│  │  ┌──────────────────────────────────────┐ │ │
│  │  │ Dure WSS Service                     │ │ │
│  │  │                                      │ │ │
│  │  │  Ports: 80 (HTTP) → 443 (HTTPS)     │ │ │
│  │  │         443 (WSS - WebSocket Secure) │ │ │
│  │  │                                      │ │ │
│  │  │  ┌────────────────────────────────┐ │ │ │
│  │  │  │ TLS Layer                      │ │ │ │
│  │  │  │ - ACME certificates (Let's     │ │ │ │
│  │  │  │   Encrypt via lego)            │ │ │ │
│  │  │  │ - Self-signed fallback         │ │ │ │
│  │  │  └────────────────────────────────┘ │ │ │
│  │  │                                      │ │ │
│  │  │  ┌────────────────────────────────┐ │ │ │
│  │  │  │ HTTP/WSS Server (smol runtime) │ │ │ │
│  │  │  │ - Static file serving          │ │ │ │
│  │  │  │ - WebSocket connections        │ │ │ │
│  │  │  │ - API endpoints                │ │ │ │
│  │  │  │ - Webhook handlers             │ │ │ │
│  │  │  └────────────────────────────────┘ │ │ │
│  │  │                                      │ │ │
│  │  │  ┌────────────────────────────────┐ │ │ │
│  │  │  │ SQLite Database                │ │ │ │
│  │  │  │ - Local file-based             │ │ │ │
│  │  │  │ - Single writer               │ │ │ │
│  │  │  │ - Optimized for small shops   │ │ │ │
│  │  │  └────────────────────────────────┘ │ │ │
│  │  └──────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### Key Characteristics

- **Single-instance only** - No horizontal scaling, no load balancer
- **SQLite backend** - File-based database, no PostgreSQL complexity  
- **Ports 80/443** - Standard HTTPS with automatic TLS via ACME
- **Target scale** - Hundreds to low thousands of products
- **Philosophy** - Simplicity for small shop owners; shops that outgrow this graduate to enterprise platforms

---

## Layer 2: Federation Architecture

### Dure Group Model

Multiple shops federate through a Chief-managed registry:

```
                    ┌─────────────────────────────────┐
                    │   Chief's Dure Group Registry   │
                    │                                 │
                    │  - Member shop domains          │
                    │  - Public keys registry         │
                    │  - Group policies               │
                    │  - Access control rules         │
                    └──────────┬──────────────────────┘
                               │
                ┌──────────────┼──────────────┐
                │              │              │
                ▼              ▼              ▼
        ┌──────────┐   ┌──────────┐   ┌──────────┐
        │ Shop A   │   │ Shop B   │   │ Shop C   │
        │ (Owner)  │   │ (Partner)│   │ (Partner)│
        └────┬─────┘   └────┬─────┘   └────┬─────┘
             │              │              │
             └──────────────┼──────────────┘
                            │
                    Partner Product
                   Metadata Sync
```

### Partnership Model

**Automatic Partnership:**
- Chief adds shop to Dure group
- Shop automatically discovers all other group members
- No per-pair approval needed
- Trust derived from group membership

**Product Metadata Replication:**

```
Shop A (Local)                    Shop B (Partner)
┌────────────────┐               ┌────────────────┐
│ Products DB    │               │ Products DB    │
│ ┌────────────┐ │   Metadata   │ ┌────────────┐ │
│ │Local       │ │   ◄──────    │ │Own         │ │
│ │Products    │ │    Sync      │ │Products    │ │
│ │(Full)      │ │              │ │(Full)      │ │
│ └────────────┘ │               │ └────────────┘ │
│ ┌────────────┐ │               │                │
│ │Partner B   │ │               │                │
│ │Metadata    │ │               │                │
│ │(Cached)    │ │               │                │
│ │- IDs       │ │               │                │
│ │- Names     │ │               │                │
│ │- Options   │ │               │                │
│ └────────────┘ │               │                │
└────────────────┘               └────────────────┘
```

**What Gets Replicated:**
- Product IDs
- Product names  
- Product options/variants
- Basic metadata for browsing

**What Stays on Partner Server:**
- Full descriptions
- Images (URLs may be cached)
- Current inventory levels
- Pricing details (may be in metadata)
- Order data (always on partner server)

### Role Hierarchy

```
Dure Chief (Group Owner)
    │
    ├─ Manages group membership
    ├─ Sets governance policies
    │  - Return policies
    │  - Shipping standards  
    │  - Quality requirements
    └─ Publishes member registry

Shop Owner (Server Owner)
    │
    ├─ Full authority over own shop
    ├─ Manages own products (full data)
    ├─ Manages own orders
    ├─ Manages own guests
    └─ Views partner products (metadata only)

Partner Shop (Other Shop Owner)
    │
    ├─ Same as Shop Owner
    ├─ Product metadata visible to partners
    └─ Orders handled on own server

Guest (Customer)
    │
    ├─ No cross-shop identity
    ├─ Authenticates per-shop
    ├─ Profile per-shop
    └─ Order history per-shop
```

---

## Layer 3: Identity & Authentication

### Two-Tier Authentication Model

```
Site-to-Site (Shop ↔ Shop)          Site-to-Guest (Shop ↔ Customer)
         │                                    │
         ▼                                    ▼
    DNS TXT Records                      OAuth + Webhooks
    Public Key Auth                      Per-Shop Sessions
```

### Site-to-Site: DNS TXT Public Key Authentication

Each shop publishes its public key in DNS for partner verification:

```
Shop A Setup:
1. Generate ed25519 key pair
   ├─ Private key: stored in SQLite (crypt_keys table)
   └─ Public key: published to DNS

2. DNS TXT Record:
   _dure-pubkey.shop-a.com TXT "ed25519:ABC123..."

3. Chief Registry Entry:
   {
     "domain": "shop-a.com",
     "pubkey_dns": "_dure-pubkey.shop-a.com",
     "verified": true,
     "joined": "2026-07-04T..."
   }
```

**Partner Verification Flow:**

```
Shop A                          Shop B                      DNS
  │                               │                          │
  │ 1. Request partner metadata   │                          │
  ├──────────────────────────────>│                          │
  │                               │                          │
  │                               │ 2. Query B's public key  │
  │                               ├─────────────────────────>│
  │                               │                          │
  │                               │ 3. Return TXT record     │
  │                               │<─────────────────────────┤
  │                               │                          │
  │ 4. Signed response            │                          │
  │   (metadata + signature)      │                          │
  │<──────────────────────────────┤                          │
  │                               │                          │
  │ 5. Query A's public key       │                          │
  ├──────────────────────────────────────────────────────────>│
  │                               │                          │
  │ 6. Return TXT record          │                          │
  │<───────────────────────────────────────────────────────────┤
  │                               │                          │
  │ 7. Verify signature with      │                          │
  │    B's public key from DNS    │                          │
  └─ ✓ Trust established          │                          │
```

### Site-to-Guest: OAuth + Webhook Authentication

**OAuth Login (Per-Shop):**

```
Guest                 Shop A WSS           OAuth Provider
  │                      │                  (Kakao/Naver/Google)
  │ 1. Visit shop-a.com  │                        │
  ├─────────────────────>│                        │
  │                      │                        │
  │ 2. Click "Login with Kakao"                   │
  ├─────────────────────>│                        │
  │                      │                        │
  │                      │ 3. Redirect to OAuth   │
  │<─────────────────────┤                        │
  │                                                │
  │ 4. Authenticate                                │
  ├───────────────────────────────────────────────>│
  │                                                │
  │ 5. Callback with code                          │
  │<───────────────────────────────────────────────┤
  │                      │                        │
  │ 6. Forward auth code │                        │
  ├─────────────────────>│                        │
  │                      │                        │
  │                      │ 7. Exchange for token  │
  │                      ├───────────────────────>│
  │                      │                        │
  │                      │ 8. Access token        │
  │                      │<───────────────────────┤
  │                      │                        │
  │ 9. Create session    │                        │
  │    (shop-a specific) │                        │
  │<─────────────────────┤                        │
```

**Session Isolation:**
- Each shop maintains separate guest database
- No session federation between shops
- Guest may have different OAuth identities on different shops
- Order history and profile are shop-specific

**Payment Webhook Authentication:**

```
Payment Gateway          Shop A WSS Server
(Portone/KakaoPay)             │
      │                        │
      │ POST /webhook/payment  │
      │ Headers:               │
      │   X-Signature: HMAC... │
      │ Body:                  │
      │   order_id, amount,    │
      │   status, timestamp    │
      ├───────────────────────>│
      │                        │
      │                        │ 1. Verify HMAC signature
      │                        │    using shared secret
      │                        │
      │                        │ 2. Validate order_id exists
      │                        │    in local database
      │                        │
      │                        │ 3. Update order status
      │                        │
      │ 200 OK                 │
      │<───────────────────────┤
```

**Webhook Security:**
- HMAC signature verification (HS256)
- Shared secret configured per payment gateway
- Idempotency checks (prevent duplicate processing)
- Timestamp validation (prevent replay attacks)

---

## Layer 4: Application Workflows

### Guest Shopping Flow (Cross-Shop Purchase)

```
Guest Browser          Shop A (Local)         Shop B (Partner)      Payment Gateway
     │                      │                        │                    │
     │ 1. Browse products   │                        │                    │
     ├─────────────────────>│                        │                    │
     │                      │                        │                    │
     │ 2. View catalog:     │                        │                    │
     │    - Shop A products │                        │                    │
     │    - Shop B metadata │                        │                    │
     │<─────────────────────┤                        │                    │
     │                      │                        │                    │
     │ 3. Click Shop B product                       │                    │
     ├─────────────────────>│                        │                    │
     │                      │                        │                    │
     │                      │ 4. Request full details│                    │
     │                      ├───────────────────────>│                    │
     │                      │   (verify via DNS TXT) │                    │
     │                      │                        │                    │
     │                      │ 5. Product details     │                    │
     │                      │<───────────────────────┤                    │
     │                      │                        │                    │
     │ 6. Show full product │                        │                    │
     │<─────────────────────┤                        │                    │
     │                      │                        │                    │
     │ 7. Add to cart       │                        │                    │
     ├─────────────────────>│                        │                    │
     │                      │                        │                    │
     │ 8. Checkout (partner item)                    │                    │
     ├─────────────────────>│                        │                    │
     │                      │                        │                    │
     │ 9. Redirect to Shop B│                        │                    │
     │<─────────────────────┤                        │                    │
     │                                                │                    │
     │ 10. Place order on Shop B                     │                    │
     ├───────────────────────────────────────────────>│                    │
     │                                                │                    │
     │                                                │ 11. Create order   │
     │                                                │     (SQLite)       │
     │                                                │                    │
     │ 12. Redirect to payment                        │                    │
     │<───────────────────────────────────────────────┤                    │
     │                                                                     │
     │ 13. Complete payment                                                │
     ├─────────────────────────────────────────────────────────────────────>│
     │                                                                     │
     │                                                │ 14. Webhook        │
     │                                                │<────────────────────┤
     │                                                │                    │
     │                                                │ 15. Update order   │
     │                                                │     status         │
     │                                                │                    │
     │ 16. Confirmation                               │                    │
     │<───────────────────────────────────────────────┤                    │
```

**Key Flow Points:**
- Guest browses **Shop A**, sees **Shop B** product metadata
- Clicking Shop B product fetches full details from **Shop B**
- Checkout redirects to **Shop B** (order owner)
- Payment webhook goes to **Shop B** (order authority)
- Shop A never stores the order (only metadata reference if tracking is desired)

### Product Metadata Sync Flow

```
Chief Registry          Shop A              Shop B              Shop C
      │                   │                   │                   │
      │ 1. Shop B joins   │                   │                   │
      │   Dure group      │                   │                   │
      │<──────────────────┤                   │                   │
      │                   │                   │                   │
      │ 2. Update member  │                   │                   │
      │    registry       │                   │                   │
      │                   │                   │                   │
      │ 3. Notify members │                   │                   │
      │   of new partner  │                   │                   │
      ├──────────────────>│                   │                   │
      ├───────────────────┼──────────────────>│                   │
      ├───────────────────┼───────────────────┼──────────────────>│
      │                   │                   │                   │
      │                   │ 4. Fetch Shop B   │                   │
      │                   │    metadata       │                   │
      │                   ├──────────────────>│                   │
      │                   │   (DNS TXT auth)  │                   │
      │                   │                   │                   │
      │                   │ 5. Product metadata                   │
      │                   │    (ID, name, options)                │
      │                   │<──────────────────┤                   │
      │                   │                   │                   │
      │                   │ 6. Cache locally  │                   │
      │                   │    (SQLite)       │                   │
```

**Sync Triggers:**
- New member joins group (via Chief notification)
- Periodic refresh (configurable interval)
- Manual refresh (shop owner action)
- Partner publishes product update (push notification via WSS)

### Data Ownership Matrix

```
┌─────────────────┬──────────┬──────────┬──────────┐
│ Data Type       │ Shop A   │ Shop B   │ Shop C   │
├─────────────────┼──────────┼──────────┼──────────┤
│ Shop A Products │ FULL ✓   │ Meta     │ Meta     │
│ Shop B Products │ Meta     │ FULL ✓   │ Meta     │
│ Shop C Products │ Meta     │ Meta     │ FULL ✓   │
├─────────────────┼──────────┼──────────┼──────────┤
│ Shop A Orders   │ FULL ✓   │ -        │ -        │
│ Shop B Orders   │ -        │ FULL ✓   │ -        │
│ Shop C Orders   │ -        │ -        │ FULL ✓   │
├─────────────────┼──────────┼──────────┼──────────┤
│ Shop A Guests   │ FULL ✓   │ -        │ -        │
│ Shop B Guests   │ -        │ FULL ✓   │ -        │
│ Shop C Guests   │ -        │ -        │ FULL ✓   │
└─────────────────┴──────────┴──────────┴──────────┘

FULL = Full data with all fields
Meta = Metadata only (ID, name, options)
-    = No data stored
```

### Chief Governance

**Chief Actions:**
```
Chief Management Console
├── Member Management
│   ├── Add shop to group
│   ├── Remove shop from group
│   ├── View member list
│   └── Verify DNS TXT records
│
├── Policy Management
│   ├── Set return policy template
│   ├── Set shipping standards
│   ├── Set quality requirements
│   └── Publish policy updates
│
└── Registry Operations
    ├── Publish member directory
    ├── Update public key registry
    └── Audit member compliance
```

**Chief does NOT:**
- Control individual product listings
- Set pricing
- Manage inventory
- Process orders
- Access guest data
- Handle payments

---

## Data Model

### Alignment with AsyncAPI Messages

The database schema maps to existing message types in `crates/asyncapi-gen/src/messages/`:
- `product.rs` - Product messages (ProductData, ProductCreateRequest, etc.)
- `order.rs` - Order messages (OrderData, OrderItem, OrderStatus, etc.)

### Database Schema (SQLite)

**Federation Tables (New):**

```sql
-- Dure Group Membership
CREATE TABLE dure_groups (
    group_id TEXT PRIMARY KEY,
    group_name TEXT NOT NULL,
    chief_domain TEXT NOT NULL,
    chief_pubkey TEXT NOT NULL,
    joined_at INTEGER NOT NULL,
    policies_json TEXT,  -- JSON: return policy, shipping standards, etc.
    last_sync INTEGER NOT NULL
);

-- Partner Shops in Group  
CREATE TABLE partner_shops (
    partner_id TEXT PRIMARY KEY,      -- Same as server_id in messages
    group_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    pubkey_dns TEXT NOT NULL,
    pubkey TEXT NOT NULL,
    verified INTEGER NOT NULL,
    last_verified INTEGER NOT NULL,
    metadata_last_sync INTEGER,
    FOREIGN KEY (group_id) REFERENCES dure_groups(group_id)
);

-- Partner Product Metadata (Cached from ProductData message)
CREATE TABLE partner_products (
    product_id TEXT NOT NULL,
    server_id TEXT NOT NULL,           -- Maps to partner_id
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    image_url TEXT,
    price_amount REAL NOT NULL,        -- From ProductPrice.amount
    price_currency TEXT NOT NULL,      -- From ProductPrice.currency
    stock INTEGER NOT NULL,            -- From ProductData.stock
    sku TEXT,
    is_available INTEGER NOT NULL,     -- From ProductData.is_available
    cached_at INTEGER NOT NULL,
    PRIMARY KEY (product_id, server_id),
    FOREIGN KEY (server_id) REFERENCES partner_shops(partner_id)
);
```

**Local Shop Tables (Maps to AsyncAPI ProductData):**

```sql
-- Local Products (Full Data - owned by this shop)
CREATE TABLE local_products (
    product_id TEXT PRIMARY KEY,
    server_id TEXT NOT NULL,           -- This shop's server_id
    name TEXT NOT NULL,                -- ProductData.name
    category TEXT NOT NULL,            -- ProductData.category
    image_url TEXT NOT NULL,           -- ProductData.image_url
    description TEXT NOT NULL,         -- ProductData.description
    price_amount REAL NOT NULL,        -- ProductPrice.amount
    price_currency TEXT NOT NULL,      -- ProductPrice.currency
    discount_percent REAL,             -- ProductPrice.discount_percent
    stock INTEGER NOT NULL,            -- ProductData.stock
    sku TEXT,                          -- ProductData.sku
    is_available INTEGER NOT NULL,     -- ProductData.is_available
    created_at INTEGER NOT NULL,       -- ProductData.created_at
    updated_at INTEGER,                -- ProductData.updated_at
    
    CHECK (is_available IN (0, 1))
);
```

**Order Tables (Maps to AsyncAPI OrderData):**

```sql
-- Local Orders (Owned by this shop)
CREATE TABLE local_orders (
    order_id TEXT PRIMARY KEY,         -- OrderData.order_id
    server_id TEXT NOT NULL,           -- OrderData.server_id (this shop)
    customer_id TEXT NOT NULL,         -- OrderData.customer_id (guest_id)
    total_amount REAL NOT NULL,        -- OrderData.total_price.amount
    total_currency TEXT NOT NULL,      -- OrderData.total_price.currency
    status TEXT NOT NULL,              -- OrderData.status (OrderStatus enum)
    notes TEXT,                        -- OrderData.notes
    channel_id TEXT,                   -- OrderData.channel_id
    created_at INTEGER NOT NULL,       -- OrderData.created_at
    updated_at INTEGER,                -- OrderData.updated_at
    
    -- Shipping address (denormalized from OrderData.shipping_address)
    shipping_recipient_name TEXT NOT NULL,
    shipping_phone TEXT NOT NULL,
    shipping_address_line1 TEXT NOT NULL,
    shipping_address_line2 TEXT,
    shipping_city TEXT NOT NULL,
    shipping_state TEXT,
    shipping_postal_code TEXT NOT NULL,
    shipping_country TEXT NOT NULL,
    
    FOREIGN KEY (customer_id) REFERENCES local_guests(guest_id),
    CHECK (status IN ('pending','processing','paid','shipped','delivered','cancelled','refunded'))
);

-- Order Items (Maps to OrderItem)
CREATE TABLE local_order_items (
    item_id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL,
    product_id TEXT NOT NULL,          -- OrderItem.product_id
    product_name TEXT NOT NULL,        -- OrderItem.product_name (snapshot)
    quantity INTEGER NOT NULL,         -- OrderItem.quantity
    unit_price_amount REAL NOT NULL,   -- OrderItem.unit_price.amount
    unit_price_currency TEXT NOT NULL, -- OrderItem.unit_price.currency
    subtotal_amount REAL NOT NULL,     -- OrderItem.subtotal.amount
    subtotal_currency TEXT NOT NULL,   -- OrderItem.subtotal.currency
    
    FOREIGN KEY (order_id) REFERENCES local_orders(order_id),
    FOREIGN KEY (product_id) REFERENCES local_products(product_id)
);
```

**Guest Tables (Per-Shop):**

```sql
-- Local Guests (No change - OAuth per shop)
CREATE TABLE local_guests (
    guest_id TEXT PRIMARY KEY,         -- Maps to customer_id in orders
    oauth_provider TEXT NOT NULL,      -- kakao/naver/google
    oauth_id TEXT NOT NULL,
    email TEXT,
    display_name TEXT,
    profile_json TEXT,
    created_at INTEGER NOT NULL,
    last_login INTEGER NOT NULL,
    UNIQUE(oauth_provider, oauth_id)
);

-- Guest Sessions (Per-Shop)
CREATE TABLE guest_sessions (
    session_id TEXT PRIMARY KEY,
    guest_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    last_activity INTEGER NOT NULL,
    FOREIGN KEY (guest_id) REFERENCES local_guests(guest_id)
);
```

### Message-to-Database Mapping

```
AsyncAPI Message          →  Database Table
─────────────────────────────────────────────────────
ProductData (local)       →  local_products
ProductData (partner)     →  partner_products (metadata only)
OrderData                 →  local_orders + local_order_items
OrderItem                 →  local_order_items
ShippingAddress           →  local_orders (denormalized columns)
ProductPrice              →  *_amount + *_currency columns
```

### Data Size Estimates (Small Shop)

```
Scenario: 1 shop in 10-member Dure group, 500 local products, 50 orders/month

┌─────────────────────────┬────────────┬─────────────┐
│ Table                   │ Row Count  │ Approx Size │
├─────────────────────────┼────────────┼─────────────┤
│ dure_groups             │ 1          │ ~1 KB       │
│ partner_shops           │ 9          │ ~10 KB      │
│ partner_products        │ 4,500      │ ~500 KB     │
│ local_products          │ 500        │ ~2 MB       │
│ local_orders            │ 600/year   │ ~100 KB     │
│ local_guests            │ 200        │ ~50 KB      │
│ guest_sessions          │ 20-50      │ ~10 KB      │
├─────────────────────────┼────────────┼─────────────┤
│ Total                   │            │ ~3 MB       │
└─────────────────────────┴────────────┴─────────────┘

SQLite handles this trivially. Even at 10x scale (5,000 local products,
50,000 partner products, 5,000 guests), database remains under 50 MB.
```

---

## Complete System Architecture

### End-to-End Deployment Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Dure Federation                                 │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Chief's Registry Server                        │  │
│  │  Domain: chief.dure-group.com                                     │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │ DNS TXT: _dure-chief.chief.dure-group.com                  │  │  │
│  │  │ Value: "ed25519:CHIEF_PUBLIC_KEY_BASE64"                   │  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │ Member Registry (Published via API)                        │  │  │
│  │  │ {                                                          │  │  │
│  │  │   "group_id": "dure-01",                                   │  │  │
│  │  │   "members": [                                             │  │  │
│  │  │     {"domain": "shop-a.com", "pubkey_dns": "..."},        │  │  │
│  │  │     {"domain": "shop-b.com", "pubkey_dns": "..."},        │  │  │
│  │  │     {"domain": "shop-c.com", "pubkey_dns": "..."}         │  │  │
│  │  │   ],                                                       │  │  │
│  │  │   "policies": {                                            │  │  │
│  │  │     "return_window_days": 14,                             │  │  │
│  │  │     "shipping_standard": "next-day",                      │  │  │
│  │  │     "quality_requirements": "..."                         │  │  │
│  │  │   }                                                        │  │  │
│  │  │ }                                                          │  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │   Shop A         │  │   Shop B         │  │   Shop C         │      │
│  │   (GCP VM)       │  │   (GCP VM)       │  │   (GCP VM)       │      │
│  ├──────────────────┤  ├──────────────────┤  ├──────────────────┤      │
│  │ Domain:          │  │ Domain:          │  │ Domain:          │      │
│  │ shop-a.com       │  │ shop-b.com       │  │ shop-c.com       │      │
│  │                  │  │                  │  │                  │      │
│  │ Ports:           │  │ Ports:           │  │ Ports:           │      │
│  │ 80 → 443 (TLS)   │  │ 80 → 443 (TLS)   │  │ 80 → 443 (TLS)   │      │
│  │ 443 (WSS)        │  │ 443 (WSS)        │  │ 443 (WSS)        │      │
│  │                  │  │                  │  │                  │      │
│  │ SQLite DB:       │  │ SQLite DB:       │  │ SQLite DB:       │      │
│  │ - Own products   │  │ - Own products   │  │ - Own products   │      │
│  │ - Own orders     │  │ - Own orders     │  │ - Own orders     │      │
│  │ - Own guests     │  │ - Own guests     │  │ - Own guests     │      │
│  │ - B+C metadata   │  │ - A+C metadata   │  │ - A+B metadata   │      │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
│           │                     │                     │                 │
│           └─────────────────────┴─────────────────────┘                 │
│                      Product Metadata Sync                              │
│                      (DNS TXT verified)                                 │
└─────────────────────────────────────────────────────────────────────────┘
                                 │
                                 │ HTTPS/WSS
                                 │
                    ┌────────────┴────────────┐
                    │                         │
              ┌─────▼──────┐          ┌──────▼──────┐
              │  Guest A   │          │  Guest B    │
              │  Browser   │          │  Browser    │
              └────────────┘          └─────────────┘
                    │                         │
                    │ OAuth Login             │ OAuth Login
                    │ (Kakao/Naver/Google)    │ (Kakao/Naver/Google)
                    │                         │
              ┌─────▼──────┐          ┌──────▼──────┐
              │ OAuth      │          │ Payment     │
              │ Provider   │          │ Gateway     │
              │            │          │ (Portone/   │
              │            │          │  KakaoPay)  │
              └────────────┘          └─────────────┘
                                             │
                                             │ Webhook
                                             │
                                      ┌──────▼──────┐
                                      │  Shop WSS   │
                                      │  Server     │
                                      │  (Direct)   │
                                      └─────────────┘
```

### Data Flow: Guest Purchases Partner Product

```
Step  Actor          Action                     Data Flow
────  ─────          ──────                     ─────────
1     Guest          Visit shop-a.com           → Shop A
2     Shop A         Serve catalog page         ← Shop A SQLite
                     - Local products (full)      (local_products +
                     - Partner products (meta)     partner_products)
3     Guest          Click Shop B product       → Shop A
4     Shop A         Fetch full details         → Shop B
                     (verify DNS TXT)             (DNS verify)
5     Shop B         Return ProductData         ← Shop B SQLite
                     (signed response)            (local_products)
6     Shop A         Display to guest           → Guest
7     Guest          Add to cart, checkout      → Shop A
8     Shop A         Redirect to Shop B         → Guest → Shop B
9     Guest          Place order on Shop B      → Shop B
10    Shop B         Create OrderData           → Shop B SQLite
                                                  (local_orders)
11    Shop B         Redirect to payment        → Guest → Payment GW
12    Guest          Complete payment           → Payment GW
13    Payment GW     Send webhook               → Shop B
                     (HMAC verified)              (webhook handler)
14    Shop B         Update order status        → Shop B SQLite
                     to 'paid'                    (UPDATE local_orders)
15    Shop B         Notify guest               → Guest
                     Order confirmation           (email/WSS)
16    Shop A         (Optional) Query status    → Shop B
                     for tracking                 (if implemented)
```

### Security Summary

```
┌───────────────────────────────────────────────────────────────┐
│                    Security Layers                             │
├───────────────────────────────────────────────────────────────┤
│                                                                │
│ 1. Transport Security                                          │
│    ├─ TLS 1.2+ (ACME certificates via Let's Encrypt)          │
│    ├─ WSS (WebSocket Secure)                                  │
│    └─ HSTS headers                                            │
│                                                                │
│ 2. Site-to-Site Authentication                                │
│    ├─ DNS TXT public key publication                          │
│    ├─ ed25519 signature verification                          │
│    ├─ Chief-mediated trust (group membership)                 │
│    └─ No shared secrets (public key crypto)                   │
│                                                                │
│ 3. Site-to-Guest Authentication                               │
│    ├─ OAuth 2.0 (Kakao/Naver/Google)                          │
│    ├─ Per-shop sessions (no federation)                       │
│    ├─ Session tokens (HTTP-only cookies)                      │
│    └─ CSRF protection                                         │
│                                                                │
│ 4. Payment Security                                            │
│    ├─ HMAC webhook verification (HS256)                       │
│    ├─ Timestamp validation (anti-replay)                      │
│    ├─ Idempotency checks                                      │
│    └─ Direct gateway → shop (no intermediary)                 │
│                                                                │
│ 5. Data Privacy                                                │
│    ├─ Guest data never shared between shops                   │
│    ├─ Orders owned by product's shop only                     │
│    ├─ Product metadata public, full data private              │
│    └─ SQLite local-only (no replication)                      │
│                                                                │
└───────────────────────────────────────────────────────────────┘
```

---

## Implementation Considerations

### Phase 1: Foundation (Weeks 1-2)
1. Database schema migrations (new tables)
2. DNS TXT record generator/verifier
3. Chief registry API client
4. Partner shop discovery mechanism

### Phase 2: Federation (Weeks 3-4)
5. Product metadata sync service
6. Partner product catalog UI
7. Cross-shop product detail fetching
8. Signature verification middleware

### Phase 3: Commerce (Weeks 5-6)
9. Cross-shop cart and checkout flow
10. Order creation on partner server
11. Payment gateway integration (existing, may need updates)
12. Webhook routing and verification

### Phase 4: Governance (Week 7)
13. Chief policy management UI
14. Group membership API
15. Compliance monitoring

### Testing Strategy
- Unit tests for signature verification
- Integration tests for product sync
- End-to-end tests for cross-shop purchase flow
- Security audit of DNS TXT verification
- Load testing (small shop scale: 100 concurrent users)

### Performance Targets
- Product metadata sync: < 5 seconds for 500 products
- Partner product detail fetch: < 500ms (cached DNS lookups)
- Checkout redirect: < 200ms
- Webhook processing: < 100ms

---

## Open Questions

1. **Chief Implementation** - Will Chief be a separate codebase or a mode of the Dure WSS server?
2. **Product Image Hosting** - Should partner product images be proxied/cached or direct-linked?
3. **Inventory Staleness** - How often should partner product metadata refresh? (Proposal: 1 hour default, configurable)
4. **Group Size Limit** - Maximum members per Dure group? (Proposal: 50 shops)
5. **Conflict Resolution** - What happens if DNS TXT verification fails? Graceful degradation or hard failure?

---

## Success Criteria

This architecture is successful if:

1. ✅ Shop owners can deploy with single command (`dure install`)
2. ✅ Joining a Dure group takes < 5 minutes
3. ✅ Product catalog includes partner shops within 10 minutes of joining
4. ✅ Guest can complete cross-shop purchase without noticing technical complexity
5. ✅ Zero shared secrets (all auth via public key crypto)
6. ✅ Guest data remains isolated per-shop (privacy preserved)
7. ✅ SQLite database stays under 50 MB for typical small shop (3 years of operation)

---

## Appendix: Existing Codebase Integration

### Files to Modify
- `mobile/src/storage/diesel_schema.rs` - Add federation tables
- `mobile/src/storage/models.rs` - Add federation models
- `mobile/src/calc/db.rs` - Add federation queries
- `mobile/src/wss/server/mod.rs` - Add partner product API endpoints
- `mobile/src/api/mod.rs` - Add Chief registry client

### Files to Create
- `mobile/src/calc/federation.rs` - Federation business logic
- `mobile/src/calc/dns_verify.rs` - DNS TXT verification
- `mobile/src/ui_tabs/federation.rs` - Federation management UI

### AsyncAPI Messages (Already Exist)
- `crates/asyncapi-gen/src/messages/product.rs` - ProductData, etc.
- `crates/asyncapi-gen/src/messages/order.rs` - OrderData, etc.

No changes needed to existing message types; database schema aligns with them.

---

**End of Design Document**
