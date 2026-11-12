# Cloudflare OAuth App Setup Guide

This guide explains how to configure a Cloudflare OAuth application for Dure Installer's DNS management features.

## Prerequisites

- Cloudflare account
- At least one domain or zone in your Cloudflare account

## Step-by-Step Setup

### 1. Create OAuth Application

1. Go to [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Navigate to **My Profile** → **API Tokens** → **Create Token**
3. Scroll down to **OAuth Applications** section
4. Click **Create Application**

### 2. Configure Application Settings

Fill in the following details:

| Field | Value |
|-------|-------|
| **Application Name** | `Dure Installer` (or any name you prefer) |
| **Redirect URLs** | `http://localhost:51082/cf_callback` |
| **Application Type** | Public Client |

**CRITICAL**: The redirect URL **MUST** be exactly `http://localhost:51082/cf_callback`
- Do not use `127.0.0.1` instead of `localhost`
- Do not change the port number
- Do not change the path `/cf_callback`

### 3. Configure Scopes

Select the following scopes (required for DNS management):

#### Account Scopes
- ✅ `offline_access` - Enable refresh tokens for automatic token renewal
- ✅ `account-dns-settings.read` - Read account DNS settings
- ✅ `account-dns-settings.write` - Write account DNS settings

#### DNS Scopes
- ✅ `dns.read` - Read DNS records
- ✅ `dns.write` - Create/update DNS records
- ✅ `dns-view.read` - Read DNS views
- ✅ `dns-view.write` - Write DNS views

#### Zone Scopes
- ✅ `zone.read` - Read zones
- ✅ `zone.write` - Create/update zones
- ✅ `zone-dns-settings.read` - Read zone DNS settings
- ✅ `zone-dns-settings.write` - Write zone DNS settings
- ✅ `zone-settings.read` - Read zone settings
- ✅ `zone-settings.write` - Write zone settings

#### Registrar Scopes (Optional - for domain registration)
- ✅ `registrar-domains.admin` - Manage domains
- ✅ `registrar-domains.read` - Read domain info

### 4. Save Client ID

After creating the application:
1. Copy the **Client ID** (e.g., `47d31be581def48e92dbce05745335c3`)
2. You don't need a Client Secret (PKCE flow doesn't require it)

### 5. Optional: Configure Environment Variable

If you want to use a custom OAuth app (instead of the default):

```bash
# In your .env file (don't commit this file!)
CLOUDFLARE_OAUTH_CLIENT_ID=your-client-id-here
```

Then rebuild the application:
```bash
cargo build --release
```

## Using OAuth in Dure Installer

1. Open Dure Installer
2. Go to **DNS** tab
3. Click **Settings Check** or **DNS Wizard**
4. Click **OAuth** button
5. Browser opens to Cloudflare authorization page
6. Review and approve the requested permissions
7. Browser redirects back to `http://localhost:51082/cf_callback`
8. Dure Installer receives authorization and exchanges for access token

## Troubleshooting

### Error: "OAuth authorization failed: access_denied"

**Cause**: You clicked "Deny" or "Cancel" on the Cloudflare authorization page.

**Solution**: Click the OAuth button again and approve the permissions.

### Error: "No code parameter in callback"

**Cause**: Redirect URI mismatch between Dure Installer and your OAuth app configuration.

**Solution**: 
1. Check your OAuth app settings in Cloudflare dashboard
2. Ensure redirect URL is exactly: `http://localhost:51082/cf_callback`
3. Update if necessary and try again

### Error: "Failed to bind to port 51082"

**Cause**: Port 51082 is already in use by another application.

**Solution**:
1. Close any other applications using port 51082
2. On Linux/macOS, check with: `lsof -i :51082`
3. On Windows, check with: `netstat -ano | findstr :51082`
4. Kill the process or restart Dure Installer

### Error: "insufficient_scope"

**Cause**: Your OAuth app is missing required scopes.

**Solution**:
1. Go to your OAuth app settings in Cloudflare dashboard
2. Add all required scopes listed in Step 3 above
3. Try the OAuth flow again

### Browser doesn't open automatically

**Cause**: System browser couldn't be launched automatically.

**Solution**:
1. Check the terminal/console output for the OAuth URL
2. Manually copy and paste the URL into your browser
3. Complete the authorization flow

### Token expires too quickly

**Cause**: Missing `offline_access` scope.

**Solution**:
1. Add `offline_access` scope to your OAuth app
2. Re-authorize to get a refresh token
3. Access token will be automatically refreshed when expired

## Security Notes

- **Never share your access token or refresh token**
- Tokens are stored securely in your system keyring (when available)
- OAuth app Client ID is public (no secret needed for PKCE flow)
- Redirect URI only accepts localhost (no remote redirects)

## Reference

- [Cloudflare OAuth Documentation](https://developers.cloudflare.com/fundamentals/api/oauth/)
- [OAuth 2.0 PKCE RFC](https://tools.ietf.org/html/rfc7636)
- Dure Installer OAuth implementation: `mobile/src/api/cloudflare/oauth.rs`
