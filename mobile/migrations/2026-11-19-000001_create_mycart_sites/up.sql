-- Create mycart_sites table for storing mycart admin settings
CREATE TABLE IF NOT EXISTS mycart_sites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_address TEXT NOT NULL UNIQUE,
    admin_id TEXT NOT NULL,
    encrypted_password TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Index for fast lookups by site_address
CREATE INDEX IF NOT EXISTS idx_mycart_sites_site_address ON mycart_sites(site_address);
