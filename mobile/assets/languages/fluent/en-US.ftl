app-title = Dure
app-title-full = Dure Installer
app-description = Distributed e-commerce platform for small shop owners.
settings = Settings
exit = Exit
debloat = Debloat
scan = Scan
apps = Apps
usage = Usage
devices = Devices:
users = Users:
select-device = Select device
all-users = All Users
refresh = Refresh
logs = Application Logs:
theme-mode = Theme Mode:
lightdark-mode = Light/Dark Mode:
contrast-level = Contrast Level:
log-level = Log Level:
ok = OK
stop = Stop
sort-by = Sort:

# Apps Tab
app-list = App List:
select-app-list = Select an app list
apps-found = ({ $count } apps)
category = Category
app-name = App Name
links = Links
install = Install
installed = Installed
uninstall = Uninstall
enable = Enable
disable = Disable
app-list-info = App List Info
package-info = Package Info
refresh-list = Refresh List
install-success = Installation Successful
uninstall-success = Uninstallation Successful
uninstall-confirm-title = Confirm Uninstall
uninstall-confirm-single = Uninstalling "{ $name }" will delete the app and its data. This action cannot be undone.
uninstall-confirm-batch = Uninstalling { $count } selected apps will delete them and their data. This action cannot be undone.
install-error = Installation Error
check-update = Check for Update
update-now = Update Now
checking-update = Checking...
update-available = Update available:
up-to-date = Up to date
update-error = Update check failed
autoupdate = Auto Update
autoupdate-desc = Automatically check for updates on startup
update-available-title = Update Available
update-available-message = A new version is available:
release-notes = Release Notes:
update-android-instruction = Click "Update Now" to open the download page in your browser. After downloading, install the APK to update the app.
update-desktop-instruction = Click "Update Now" to download and install the update automatically.

# Debloat Tab
all = All ({ $enabled }/{ $total })
recommended = Recommended ({ $enabled }/{ $total })
advanced = Advanced ({ $enabled }/{ $total })
expert = Expert ({ $enabled }/{ $total })
unsafe = Unsafe ({ $enabled }/{ $total })
unknown = Unknown ({ $enabled }/{ $total })
label-recommended = Recommended
label-advanced = Advanced
label-expert = Expert
label-unsafe = Unsafe
label-unknown = Unknown
show-only-enabled = Show only enabled
hide-system-app = Hide system app
filter = Filter:
filter-hint = Search packages...
show-only-installable = Show only installable
disable-github-install = Disable GitHub installation
col-package-name = Package Name
col-debloat-category = Debloat Category
col-runtime-permissions = Runtime Permissions
col-enabled = Enabled
col-stalkerware = Stalkerware
col-install-reason = Install Reason
col-tasks = Tasks
stalkerware = STALKERWARE
not-listed = NOT LISTED
stalkerware-info = This package is identified as stalkerware
no-packages-loaded = No packages loaded. Please select a device from the dropdown.
deselect-all = Deselect All
selected-count = Selected: { $count }
uninstall-selected = Uninstall Selected ({ $count })
disable-selected = Disable Selected ({ $count })
enable-selected = Enable Selected ({ $count })

# Scan Tab
virustotal-filter = VirusTotal Filter:
scanning-packages = Scanning packages...
malicious = Malicious ({ $enabled }/{ $total })
malicious-ignored = Malicious-Ignored ({ $enabled }/{ $total })
suspicious = Suspicious ({ $enabled }/{ $total })
safe = Safe ({ $enabled }/{ $total })
no-specific-threat = No Specific Threat ({ $enabled }/{ $total })
not-scanned = Not Scanned ({ $enabled }/{ $total })
hybrid-analysis-filter = Hybrid-Analysis Filter:
col-izzy-risk = IzzyRisk
col-virustotal = VirusTotal
col-hybrid-analysis = HybridAnalysis
scan-not-initialized = Not initialized
scan-not-scanned = Not scanned
scan-scanning = Scanning... ({ $scanned }/{ $total })
scan-error = Error
scan-error-msg = Error: { $message }
scan-skip = skip
scan-404 = 404
scan-malicious = malicious { $count }/{ $total }
scan-suspicious = suspicious { $count }/{ $total }
scan-clean = clean { $count }/{ $total }
scan-no-results = No results
ha-malicious = malicious
ha-malicious-score = malicious ({ $score })
ha-malicious-tags = malicious ({ $tags })
ha-suspicious = suspicious
ha-suspicious-score = suspicious ({ $score })
ha-suspicious-tags = suspicious ({ $tags })
ha-whitelisted = whitelisted
ha-whitelisted-score = whitelisted ({ $score })
ha-whitelisted-tags = whitelisted ({ $tags })
ha-no-result = no-result
ha-no-specific-threat = no specific threat
ha-no-specific-threat-score = no specific threat ({ $score })
ha-no-specific-threat-tags = no specific threat ({ $tags })
ha-skipped = skip
ha-rate-limited = rate limited
ha-submitted = submitted
ha-pending = pending ({ $jobid }...)
ha-pending-analysis = pending analysis
ha-analysis-error = analysis error
ha-upload-error = upload error
ha-404 = 404 Not Found
ha-file-too-large = File too large: { $size }
ha-file-too-large-default = File too large (>100 MB)
ha-pull-failed = Pull failed: File not found
ha-temp-dir-error = Temp dir error
ha-wait-hours = { $text } (wait { $hours }h{ $mins }m)
ha-wait-mins = { $text } (wait { $mins }m)
ha-wait-less-than-min = { $text } (wait <1m)
refresh-scan = Re-scan this package
izzyrisk-calculation = IzzyRisk Calculation:
calculating-risk-scores = Calculating risk scores...

# Usage Tab
usage-control = Usage Control
no-usage-stats = No usage statistics available. Select a device to view usage stats.
usage-statistics = Usage Statistics:

# Settings
display-size = Display Size:
color-mode = Color Mode:
light-mode = ☀️ Light
auto-mode = 🌗 Auto
dark-mode = 🌙 Dark
contrast = Contrast:
contrast-high = High
contrast-medium = Medium
contrast-normal = Normal
allow-unsafe-app-remove = Allow unsafe app uninstall/disable
virustotal-api-key = VirusTotal(4/min) API Key:
get-api-key = Get API Key
allow-virustotal-upload = Allow VirusTotal file upload
virustotal-upload-desc = (Upload files not found in VirusTotal for analysis)
hybridanalysis-api-key = HybridAnalysis(200/min) API Key:
allow-hybridanalysis-upload = Allow Hybrid Analysis file upload
hybridanalysis-upload-desc = (Upload files not found in Hybrid Analysis for scanning)
hybridanalysis-tag-ignorelist = HybridAnalysis Tag Ignorelist:
google-play-renderer = Google Play Renderer
google-play-renderer-desc = (Fetch and display app info from Google Play Store)
fdroid-renderer = F-Droid Renderer
fdroid-renderer-desc = (Fetch and display app info from F-Droid for non-system apps)
apkmirror-renderer = APKMirror Renderer
apkmirror-renderer-desc = (Fetch and display app info from APKMirror)
rendering-google-play = Google Play
rendering-fdroid = F-Droid
rendering-apkmirror = APKMirror
apkmirror-email = APKMirror Email:
email-hint = your@email.com
apkmirror-name = APKMirror Name:
name-hint = Your Name (optional)
apkmirror-auto-upload = APKMirror Auto-Upload
apkmirror-auto-upload-desc = (Auto-upload APKs when device version is newer than APKMirror)
invalidate-cache = 404 Invalidate Cache
invalidate-cache-desc = (Clear cached apps and scan results)
show-logs = Show Logs:
show = Show
cancel = Cancel
save = Save
flush = Flush

# Status & Errors
status-checking-api = Checking API
status-pulling-file = Pulling file
status-uploading = Uploading
error-invalid-package-id = Invalid package ID (less than 2 domain levels)
error-timeout-rate-limit = Timeout waiting for rate limit
error-email-not-configured = Email not configured
error-rate-limit-reached = Rate limit reached (429)
error-app-not-found = App not found

# GUI Descriptions
debloat-description = Debloat android applications with Universal Android Debloater list:
loading-packages = Loading package details...
batch-uninstall = Batch Uninstall
uninstalling-packages = Uninstalling packages...
batch-disable = Batch Disable
disabling-packages = Disabling packages...
batch-enable = Batch Enable
enabling-packages = Enabling packages...
scan-description = Scan viruses with virustotal and hybridanalysis:
set-api-keys = Please set virustotal or hybridanalysis apikey in settings.
apps-description = Lists of Foss applications :
usage-description = Application Usage :
language = Language:
font = Font:
text-style = Text Style:

# Disclaimer Dialog
disclaimer-title = ⚠ Warning
disclaimer-no-user-data = • This app does not operate over user files or any user data.
disclaimer-uninstall-warning = • Uninstall action will delete app with userdata for the app. Make careful to uninstall app.

close = Close

# About Dialog
about = About
about-description = Remove bloat apps selectively, scan installed apps with VirusTotal/HybridAnalysis, and install FOSS apps through Obtainium.
about-version = Version
about-website = Website
about-credits = Credits
about-reference-projects = Reference Projects
about-rust-libraries = Rust Libraries

# Clipboard
clipboard-copy = Copy
clipboard-paste = Paste

# Tray Menu (Desktop)
tray-show-app = Show App
tray-cache-dir = Open Cache Directory
tray-refresh-stores = Refresh Store Listings
tray-view-orders = View Recent Orders
tray-quit = Quit

# Main UI (dure.rs)
status-fetching-all = Fetching store listings and product data...
status-fetching-products = Fetching products...
status-downloading-history = Downloading order history...
status-loading-page = Loading more products (page { $page })...
status-loading-details = Loading product details...
product-title-label = Product: { $title }
product-price-label = Price: { $price }
button-add-to-cart = Add to Cart
button-buy-now = Buy Now
button-more-info = More Info
dialog-loading-products = Loading Products
status-filter = Product Filter
status-all = All
status-favorites = Favorites
status-on-sale = On Sale
status-new-arrivals = New Arrivals
switch-favorite = Favorite
switch-blacklist = Blacklist

# Navigation Tabs (dure.rs)
tab-platform = Platform
tab-server = Server
tab-ssh = SSH
tab-domains = Domains
tab-site = Site
tab-roles = Roles
tab-members = Members
tab-channel = Channel
tab-dm = DM
tab-products = Products
tab-orders = Orders
tab-email = Email
tab-client = Client

# Profile Selector
profile = Profile:
none = None
create-profile = + Create New Profile
delete-profile = Delete Profile
close-profile = Close Profile

# Profile Login Dialog
profile-login-title = Login Profile
profile-login-heading = Enter Profile Password
password = Password:
password-hint = Enter password

# Profile Create Dialog
profile-create-title = Create New Profile
profile-create-heading = Create a New Profile
profile-name = Profile Name:
profile-name-hint = Enter profile name (a-z, A-Z, 0-9, -, _)
password-confirm = Confirm Password:
password-confirm-hint = Confirm password
create = Create

# Delete Platform Dialog
delete-platform-title = Delete Platform
delete-platform-heading = Confirm Platform Deletion
delete = Delete

# Add Platform Dialog
add-platform-title = Add Platform
add-platform-heading = Add Cloud Platform
add = Add

# Re-authenticate Platform Dialog
reauth-platform-title = Re-authenticate Platform
reauth-platform-heading = Re-authenticate to Google Cloud
reauth = Re-authenticate

# DNS Tab
dns-main-heading = DNS Domains
dns-main-desc = Manage DNS domains with Cloudflare integration.
dns-button-add = Add DNS
dns-button-add-tooltip = Add new DNS with automated Cloudflare setup
dns-main-empty = No DNS domains configured. Click 'Add DNS' to get started.

# DNS Table
dns-table-col-dns = DNS
dns-table-col-operations = Operations

# DNS Operation Buttons
dns-button-check-ns = Check NS
dns-button-check-ns-tooltip = Check nameserver records
dns-button-check-settings = Check Settings
dns-button-check-settings-tooltip = Verify DNS records
dns-button-https-check = HTTPS Check
dns-button-https-check-tooltip = Test HTTPS connection
dns-button-delete = Delete
dns-button-delete-tooltip = Remove this DNS domain

# DNS NS Check Dialog
dns-dialog-ns-title = NS Server Check
dns-dialog-ns-querying = Querying NS records...
dns-dialog-ns-found = Found { $count } NS record(s)
dns-dialog-ns-not-found = ✗ No NS records found
dns-error-parse-response = Failed to parse DNS response: { $error }
dns-error-query-failed = DNS query failed: { $error }

# DNS Settings Dialog
dns-dialog-settings-title = DNS Settings Check
dns-dialog-settings-checking = Checking all DNS records...
dns-dialog-settings-verification = DNS Record Verification
dns-dialog-settings-col-type = Type
dns-dialog-settings-col-name = Name
dns-dialog-settings-col-expected = Expected
dns-dialog-settings-col-actual = Actual
dns-dialog-settings-col-status = Status
dns-dialog-settings-not-found = (not found)
dns-button-set-records = Set Correct Records
dns-button-set-records-note = (Not implemented - requires Cloudflare API token)
dns-status-not-implemented = ⚠ Cloudflare API check not implemented
dns-status-missing-a-record = Missing: A record for root domain
dns-status-missing-cname = Missing: A/CNAME for www subdomain

# DNS HTTPS Check Dialog
dns-dialog-https-title = HTTPS Connection Check
dns-dialog-https-checking = Checking DNS resolution...
dns-dialog-https-ok = ✓ Resolves Correctly
dns-dialog-https-mismatch = ✗ Resolution Mismatch
dns-dialog-https-resolved-ip = Resolved IP:
dns-dialog-https-expected-ip = Expected IP:
dns-dialog-https-matches-vm = Matches VM:
dns-dialog-https-yes = Yes
dns-dialog-https-no = No
dns-dialog-https-na = N/A
dns-dialog-https-error-prefix = Error:

# DNS Drawer Tabs
dns-drawer-tab-status = Status
dns-drawer-tab-logs = Logs
dns-drawer-tab-operations = Operations

# DNS Wizard Dialog - Step Headers
dns-wizard-step-oauth = 1. OAuth
dns-wizard-step-domain = 2. Domain
dns-wizard-step-vm = 3. VM Config
dns-wizard-step-confirm = 4. Confirm

# DNS Wizard - OAuth Step
dns-wizard-oauth-title = Connect Cloudflare Account
dns-wizard-oauth-desc = Connect your Cloudflare account to manage DNS:
dns-wizard-oauth-button = Open OAuth Screen
dns-wizard-oauth-account-id = Account ID:
dns-wizard-oauth-connected = Account connected
dns-wizard-oauth-connected-email = Connected: { $email }
dns-wizard-oauth-verified = API connection verified
dns-wizard-oauth-token-verified = API token verified
dns-wizard-oauth-fetching = Fetching account info...
dns-wizard-oauth-auto-account = Auto-populated Account ID: { $id } ({ $name })
dns-wizard-oauth-account-info = Account: { $email } (ID: { $id })
dns-wizard-oauth-error = Error: OAuth failed: { $error }
dns-wizard-oauth-error-token = Error: Invalid API token
dns-wizard-oauth-error-browser = Error: Failed to open browser: { $error }
dns-wizard-oauth-loading = Loading: Waiting for OAuth authorization...
dns-wizard-oauth-no-token = No OAuth token available

# DNS Wizard - Domain Step
dns-wizard-domain-register = I want to register domain
dns-wizard-domain-have = I have domain
dns-wizard-domain-label = Domain:
dns-wizard-domain-checking = Checking availability: { $domain }
dns-wizard-domain-loading = Loading domains...
dns-wizard-domain-loaded = Loaded { $count } domains
dns-wizard-domain-adding = Adding domain...
dns-wizard-domain-added = Added domain: { $domain }
dns-wizard-domain-connecting = Connecting existing domain: { $domain }
dns-wizard-domain-checking-ns = Checking nameservers...
dns-wizard-domain-ns-verified = Nameservers verified (stub)
dns-wizard-domain-error-fetch = Error: Failed to fetch domains: { $error }
dns-wizard-domain-error-add = Error: Failed to add domain: { $error }
dns-wizard-domain-no-selection = No domain selected

# DNS Wizard - VM Config Step
dns-wizard-vm-no-platforms = No platforms available. Please add a platform in Platform tab first.
dns-wizard-vm-default-settings = Default settings:
dns-wizard-vm-no-ip = No VM IP available
dns-wizard-vm-no-zone = No zone ID available
dns-wizard-vm-dns-missing = DNS records missing or incorrect - setup required
dns-wizard-vm-dns-exists = DNS records already exist (no changes needed)
dns-wizard-vm-dns-created = DNS records created or updated successfully
dns-wizard-vm-dns-verified = DNS records verified - all settings correct
dns-wizard-vm-error-setup = Error: Failed to setup DNS records: { $error }
dns-wizard-vm-error-verify = Error: Verification failed: { $error }
dns-wizard-vm-check-failed = Check failed: { $error }

# SSH Tab
ssh-main-heading = SSH Hosts
ssh-main-desc = Manage SSH host connections and services.
ssh-button-add = Add SSH Host

# SSH Table
ssh-table-col-host = Host
ssh-table-col-operations = Operations

# SSH Operation Buttons - Row 1
ssh-button-refresh = Refresh
ssh-button-refresh-tooltip = Refresh SSH host status
ssh-button-ssh-check = SSH Check
ssh-button-ssh-check-tooltip = Test SSH connection
ssh-button-edit = Edit
ssh-button-edit-tooltip = Edit SSH host configuration
ssh-button-delete = Delete
ssh-button-delete-tooltip = Delete SSH host

# SSH Operation Buttons - Row 2 (Base & Docker)
ssh-button-check-base = Check Base
ssh-button-check-base-tooltip = Check base system packages
ssh-button-install-base = Install Base
ssh-button-install-base-tooltip = Install required base packages
ssh-button-check-docker = Check Docker
ssh-button-check-docker-tooltip = Check Docker installation
ssh-button-install-docker = Install Docker
ssh-button-install-docker-tooltip = Install Docker on remote host
ssh-button-remove-docker = Remove Docker
ssh-button-remove-docker-tooltip = Uninstall Docker from remote host

# SSH Operation Buttons - Row 3 (Dure)
ssh-button-check-dure = Check Dure
ssh-button-check-dure-tooltip = Check Dure service status
ssh-button-install-dure = Install Dure
ssh-button-install-dure-tooltip = Install Dure service
ssh-button-remove-dure = Remove Dure
ssh-button-remove-dure-tooltip = Uninstall Dure service

# SSH Add Host Dialog
ssh-dialog-add-title = Add SSH Host
ssh-dialog-add-heading = Configure a new SSH host connection:
ssh-dialog-add-host-label = Host (IP or domain):
ssh-dialog-add-port-label = Port:
ssh-dialog-add-use-password = Use password authentication
ssh-dialog-add-password-label = Password:
ssh-dialog-add-use-key = Use private key authentication
ssh-dialog-add-key-path-label = Private key path:

# SSH Edit Host Dialog
ssh-dialog-edit-title = Edit SSH Host
ssh-dialog-edit-heading = Edit SSH host connection:

# SSH Drawer Tabs
ssh-drawer-tab-status = Status
ssh-drawer-tab-logs = Logs
ssh-drawer-tab-operations = Operations
ssh-drawer-tab-host = Host
ssh-drawer-tab-docker = Docker
ssh-drawer-tab-dure = Dure

# SSH Drawer Status Tab
ssh-drawer-status-ip = IP
ssh-drawer-status-ssh = SSH
ssh-drawer-status-base = Base
ssh-drawer-status-docker = Docker
ssh-drawer-status-dure = Dure
ssh-drawer-status-connected = Connected
ssh-drawer-status-disconnected = Disconnected
ssh-drawer-status-not-checked = Not checked
ssh-drawer-status-installed = Installed
ssh-drawer-status-not-installed = Not Installed
ssh-drawer-status-failed = Failed ({ $error })

# SSH Drawer Logs Tab
ssh-drawer-logs-heading = SSH Host Logs
ssh-drawer-logs-col-date = Date
ssh-drawer-logs-col-operation = Operation
ssh-drawer-logs-col-system = System
ssh-drawer-logs-col-status = Status
ssh-drawer-logs-col-duration = Duration
ssh-drawer-logs-col-error = Error

# SSH Drawer Host Tab
ssh-drawer-host-heading = Host Information
ssh-drawer-host-refresh = Refresh

# SSH Drawer Docker Tab
ssh-drawer-docker-heading = Docker Containers
ssh-drawer-docker-refresh = Refresh

# SSH Drawer Dure Tab
ssh-drawer-dure-heading = Dure (mycart) Status
ssh-drawer-dure-refresh = Refresh

# SSH Error Messages
ssh-error-load-config = Failed to load config: { $error }
ssh-error-save-config = Failed to save config: { $error }
ssh-error-host-exists = Host { $host } already exists
ssh-error-invalid-port = Invalid port '{ $port }': { $error }
ssh-error-no-profile = No active profile - please select or create a profile first
ssh-error-wasm-not-supported = SSH management not available on WASM

# Common Dialog Buttons (reuse existing where possible)
button-add = Add
button-save = Save
button-cancel = Cancel
button-close = Close
button-ok = OK
button-bind = Bind
button-next = Next
button-next-arrow = Next →
button-back-configure = ← Back to Configure
button-create-server = Create Server
button-manage-ips = Manage IPs in GCP Console
button-add-my-ip = Add My IP
button-remove-ip = Remove IP...
button-flush-ips = Flush All IPs
button-remove = Remove
button-refresh-cache = Refresh Cache
button-clear-expired = Clear Expired
button-encrypt = Encrypt
button-decrypt = Decrypt
button-load = Load
button-delete-selected = Delete Selected
button-add-site = Add Site
button-add-domain = Add Domain
button-add-record = Add Record
button-nameservers = Nameservers
button-add-ns-provider = Add Nameserver Provider
button-connect-gcloud = Connect to Google Cloud
button-query = Query
button-search = Search
button-setup-dns = Setup DNS
button-star = Star
button-like = Like
button-settings = Settings
button-share = Share
button-featured = Featured
button-download = Download
button-long-tooltip = Long Tooltip

# Platform Tab

## Main UI
platform-main-heading = Cloud Platforms
platform-main-desc = Manage cloud service platforms GCP for deployment and hosting.
platform-button-add = Add Platform
platform-main-empty = No platforms configured. Click 'Add Platform' to get started.

## Table
platform-table-col-project = Project
platform-table-col-operations = Operations

## Operation Buttons
platform-button-refresh = Refresh
platform-button-refresh-tooltip = Refresh platform data
platform-button-reauth = Re-auth
platform-button-reauth-tooltip = Re-authenticate to Google Cloud
platform-button-billing = Billing
platform-button-billing-tooltip = Estimated Billing
platform-button-delete = Delete
platform-button-delete-tooltip = Delete Platform

# Platform tab - Operations
platform-button-firewall = Firewall
platform-button-restart = Restart
platform-button-del-vm = Del VM

# Platform tab - Add dialog
platform-add-configure = Configure a new cloud platform:
platform-add-type = Type:
platform-add-project = Project:
platform-add-project-id = New Project ID:
platform-add-oauth-waiting = Waiting for authorization...
platform-add-oauth-complete = Please complete the OAuth flow in your browser.
platform-add-oauth-connect = Connect to Google Cloud
platform-add-oauth-url = OAuth URL (copy to browser if needed):

# Platform tab - Progress
platform-progress-heading = Initialization Progress
platform-progress-platform = Platform:

# Platform tab - Delete VM dialog
platform-delete-vm-confirm = Confirm Deletion
platform-delete-vm-question = Are you sure you want to delete VM '{$name}'?
platform-delete-vm-zone = Zone: {$zone}
platform-delete-vm-select = Select VM to Delete
platform-delete-vm-none = No VMs found in this project.
platform-delete-vm-found = Found {$count} VM(s):
platform-delete-vm-yes = Yes, Delete
platform-delete-vm-cancel = Cancel
platform-delete-vm-next = Next
platform-delete-vm-manage-ips = Manage IPs in GCP Console

# Platform tab - Select Project dialog
platform-select-project-heading = Select Project
platform-select-project-none = No projects found
platform-select-project-button = Select

# Platform tab - Drawer
platform-drawer-no-vms = This platform has no VMs configured.

# Platform tab - Billing dialog
platform-billing-heading = Monthly Total Cost (Last 3 Months)
platform-billing-project-id = Project ID:
platform-billing-loading = Loading billing data...
platform-billing-no-table = BigQuery billing export table does not exist yet.
platform-billing-error = Error:
platform-billing-help = To enable BigQuery billing export:
platform-billing-no-data = No billing data found for the current month.

## Platform Drawer
platform-drawer-status-refreshed = Refreshed
platform-drawer-status-ip = IP
platform-drawer-status-no-ip = No external IP
platform-drawer-status-updating = Updating...
platform-drawer-status-firewall = Firewall
platform-drawer-status-ssh = SSH
platform-drawer-status-no-vm = No VM created
platform-drawer-status-no-project = No project selected
platform-drawer-status-vm = VM
platform-drawer-status-project = Project

platform-drawer-menu-ssh = SSH
platform-drawer-menu-copy-ssh = Copy SSH Command
platform-drawer-menu-copy-key = Copy Private Key
platform-drawer-menu-copy-ip = Copy IP Address

platform-drawer-ssh-not-found = Not found in keyring
platform-drawer-ssh-key-label = SSH Key

platform-drawer-time-just-now = just now
platform-drawer-time-min-ago = {$minutes} min ago
platform-drawer-time-hours-ago = {$hours} hours ago
platform-drawer-time-days-ago = {$days} days ago

platform-drawer-logs-title = Operation History
platform-drawer-logs-level-all = All
platform-drawer-logs-level-debug = DEBUG
platform-drawer-logs-level-info = INFO
platform-drawer-logs-level-warn = WARN
platform-drawer-logs-level-error = ERROR

## SSH Tab & Drawer
ssh-operation-refresh = Refresh
ssh-operation-ssh-check = SSH Check
ssh-operation-base-check = Base Check
ssh-operation-install-base = Install Base
ssh-operation-docker-check = Docker Check
ssh-operation-install-docker = Install Docker
ssh-operation-remove-docker = Remove Docker
ssh-operation-dure-check = Dure Check
ssh-operation-install-dure = Install Dure
ssh-operation-remove-dure = Remove Dure

ssh-status-checking = Checking...
ssh-status-installing = Installing...
ssh-status-installed = Installed ({$version})
ssh-status-failed = Failed ({$error})

ssh-host-info-title = Host Information
ssh-host-info-os = OS: {$os}
ssh-host-info-uptime = Uptime: {$uptime}
ssh-host-info-ip = External IP: {$ip}
ssh-host-info-load = Load: {$load}
ssh-host-info-memory = Memory: {$memory}
ssh-host-info-disk = Disk: {$disk}

ssh-logs-title = SSH Host Logs
ssh-logs-network-title = Network Log (last 100 lines)

ssh-op-last-operation = Last Operation
ssh-op-operation = Operation
ssh-op-completed = {$operation} completed ({$seconds}s ago)
ssh-op-failed = {$operation} failed
ssh-op-in-progress = {$operation} ({$seconds}s)


dns-check-ns-title = NS Check
dns-check-ns-found = Found {$count} NS record(s)
dns-check-ns-none = No NS records found
dns-check-ns-error = DNS query failed: {$error}
dns-check-ns-parse-error = Failed to parse DNS response: {$error}

dns-check-settings-title = Settings Check
dns-check-settings-root = A record for root domain
dns-check-settings-www = A/CNAME record for www subdomain
dns-check-settings-cloudflare = Cloudflare API check not yet implemented - needs zone_id and API token

dns-check-https-title = HTTPS Check
dns-check-https-success = HTTPS connection successful
dns-check-https-failed = HTTPS connection failed: {$error}

dns-wizard-step1-title = Step 1: Cloudflare Authentication
dns-wizard-step1-desc = Connect your Cloudflare account to manage DNS:
dns-wizard-step1-oauth-wait = Waiting for OAuth authorization...
dns-wizard-step1-oauth-complete = Please complete the authorization in your browser.
dns-wizard-step1-oauth-redirect = You will be redirected to Cloudflare to authorize access.
dns-wizard-step1-connected = Connected: {$email}
dns-wizard-step1-account-connected = Account connected
dns-wizard-step1-account-verified = API connection verified
dns-wizard-step1-account-verifying = Verifying API connection...
dns-wizard-step1-account-id = Account ID:
dns-wizard-step1-account-required = Warning: Account ID required to continue

dns-wizard-button-close = Close
dns-wizard-button-next = Next
dns-wizard-button-previous = Previous
dns-wizard-button-oauth = Open OAuth Screen
dns-wizard-waiting = Waiting for operations...

dns-wizard-progress-oauth-success = Success: OAuth successful
dns-wizard-progress-oauth-error = Error: OAuth failed: {$error}
dns-wizard-progress-loading = Loading: Waiting for OAuth authorization...

# Platform Tab
platform-button-add-vm = Add VM
platform-button-add-vm-tooltip = Add VM
platform-button-bind-vm = Bind VM
platform-button-bind-vm-tooltip = Bind VM to Platform

platform-drawer-ssh-copy-ip = Copy IP Address

## Platform Drawer Tabs
platform-drawer-tab-status = Status
platform-drawer-tab-logs = Logs
platform-drawer-tab-operations = Operations

## Drawer Logs Tab
platform-drawer-logs-heading = Project Logs
platform-drawer-logs-level = Level:
platform-drawer-logs-loading = Loading logs...
platform-drawer-logs-no-project = No project selected
platform-drawer-logs-empty = No logs available

## Drawer Operations Tab
platform-drawer-ops-heading = Operation History
platform-drawer-ops-loading = Loading operations...
platform-drawer-ops-empty = No operations logged
platform-drawer-ops-col-date = Date
platform-drawer-ops-col-operation = Operation
platform-drawer-ops-col-system = System
platform-drawer-ops-col-status = Status
platform-drawer-ops-col-duration = Duration
platform-drawer-ops-col-error = Error

platform-drawer-ops-status-running = ⏳ Running
platform-drawer-ops-status-success = ✓ Success
platform-drawer-ops-status-failed = ✗ Failed

## Bind VM Dialog
platform-dialog-bind-title = Bind VM
platform-dialog-bind-heading = Bind VM to Platform
platform-dialog-bind-loading = Loading VMs from GCP...
platform-dialog-bind-no-vms = No VMs found
platform-dialog-bind-select = Select VM to bind ({ $count })

## Error Messages
platform-error-no-profile = No active profile - please select or create a profile first
platform-error-read-config = Failed to read config: { $error }
platform-error-save-config = Failed to save config: { $error }
platform-error-parse-config = Failed to parse config as V1 or V2: V2 error: { $v2_error }, V1 error: { $v1_error }

## Platform Drawer - Additional
platform-drawer-status-email = Email
platform-drawer-status-project-singular = project
platform-drawer-status-project-plural = projects
platform-drawer-status-project-unknown = ? projects

## Billing Dialog
platform-billing-title = Estimated Billing
platform-billing-monthly-total = Monthly Total Net Cost (including all credits) - { $count } record(s)

## SSH Drawer - Status Tab
ssh-drawer-status-operation = Operation
ssh-drawer-status-last-operation = Last Operation
ssh-op-completed-ago = { $operation } completed ({ $seconds }s ago)
ssh-op-failed-text = { $operation } failed
ssh-op-in-progress-text = { $operation } ({ $seconds }s)

## SSH Drawer - Tab Placeholders
ssh-host-placeholder = Click 'Check Base' to retrieve system status
ssh-docker-placeholder = Click 'Check Docker' to retrieve container status
ssh-dure-placeholder = Click 'Check Dure' to retrieve compose logs

## Operations Table (used in Platform, SSH, DNS drawers)
ops-col-date = Date
ops-col-operation = Operation
ops-col-system = System
ops-col-status = Status
ops-col-duration = Duration
ops-col-error = Error
ops-no-operations = No operations yet

## DNS Status Tab Attributes
dns-status-domain = Domain
dns-status-provider = Provider
dns-status-platform-binding = Platform Binding
dns-status-platform-id = Platform ID
dns-status-platform = Platform
dns-status-vm-name = VM Name
dns-status-vm-ip = VM IP
dns-status-check-status = Check Status
dns-status-ns-check = NS Check
dns-status-settings-check = Settings Check
dns-status-https-check = HTTPS Check
dns-status-not-bound = Not bound
dns-status-not-checked = Not checked
dns-status-using-cloudflare = ✓ Using Cloudflare - { $nameservers }
dns-status-not-using-cloudflare = ✗ Not using Cloudflare
dns-status-all-records-configured = ✓ All records configured
dns-status-missing-records = ✗ { $count } missing records - { $list }
dns-status-resolves-to-vm = ✓ Resolves to { $ip } (VM IP)
dns-status-resolves-correctly = ✓ Resolves correctly
dns-status-resolves-mismatch = ✗ Resolves to { $resolved } (expected { $expected })
dns-status-not-resolve-to-vm = ✗ Does not resolve to VM
dns-logs-heading = DNS Operation Logs

## DNS Operations Column - Refresh Button
dns-button-refresh = Refresh
dns-button-refresh-tooltip = Run all DNS checks (NS, Settings, HTTPS)

## DNS Settings Check Dialog
dns-settings-dialog-title = DNS Settings Check
dns-settings-status-heading = DNS Configuration Status
dns-settings-all-ok = ✓ All DNS records are configured correctly
dns-settings-incomplete = ⚠ DNS configuration incomplete
dns-settings-missing-label = Missing records:
dns-settings-error = Error: {$error}
dns-settings-auth-button = Authenticate with Cloudflare
dns-settings-setup-prompt = You can setup these DNS records automatically:
dns-settings-setup-button = Setup DNS Records
dns-settings-checking = Checking DNS settings...
dns-settings-op-log-label = Operation Log:

## Logs Renderer
logs-no-logs-yet = No logs yet

