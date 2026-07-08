# GCP Server Setup Dialog - Image Selection & Workflow Optimization

**Date:** 2026-07-07  
**Status:** Approved  
**Author:** Claude (with user collaboration)

## Overview

Enhance the GCP Server Setup dialog to streamline VM creation from the Platform tab by:
1. Skipping account/project selection steps when opened from "Add VM" button (context already known)
2. Adding source image selection with auto-fetched Debian/Ubuntu images from GCP public projects
3. Adding configurable disk size parameter
4. Implementing automatic retry with exponential backoff for image loading

## Goals

1. **Reduce friction** - Skip redundant steps when platform context is already selected
2. **Provide choice** - Let users select specific OS images instead of hardcoded defaults
3. **Show recency** - Display creation dates so users can choose recent, secure images
4. **Graceful degradation** - Auto-retry with fallback when API calls fail
5. **Maintain consistency** - Follow existing async patterns (`poll_promise`) in codebase

## Non-Goals

- Supporting Windows or other OS families (Debian/Ubuntu only for now)
- ARM64 architecture support (x86_64 only)
- Image caching across application sessions (only within dialog instance)
- Custom image upload functionality

## Context

### Current Behavior

The GCP Server Setup wizard has 5 steps:
1. **Connect Account** - OAuth login or select existing account
2. **Select Project** - Choose or create GCP project
3. **Configure Server** - Name, region, zone, machine type
4. **Create** - VM creation with progress
5. **Complete** - Show connection details

When user clicks "Add VM" button in Platform tab operations column, they've already selected a platform row containing account and project. Steps 1-2 are redundant.

The wizard currently uses hardcoded image: `projects/debian-cloud/global/images/family/debian-13` and fixed disk size: `10 GB`.

### User Flow Trigger

```
Platform Tab
└─ Platform Row (GCP account + project selected)
   └─ Operations Column
      └─ [Add VM] button clicked
         └─ GCP Server Setup dialog opens
```

## Architecture

### Component Changes

#### `mobile/src/api/gcp/compute.rs`

**New Types:**

```rust
/// Image list response from GCP API
#[derive(Debug, Deserialize)]
pub struct ImageList {
    #[serde(default)]
    pub items: Vec<Image>,
}

/// GCP Compute Engine image metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub name: String,
    pub description: Option<String>,
    pub self_link: String,              // Full resource URL for InstanceRequest
    pub creation_timestamp: String,      // RFC3339 format
    pub architecture: Option<String>,    // "X86_64" or "ARM64"
    pub family: Option<String>,          // "debian-13", "ubuntu-2404-lts"
    pub deprecated: Option<DeprecatedStatus>,
}

/// Deprecation status
#[derive(Debug, Clone, Deserialize)]
pub struct DeprecatedStatus {
    pub state: Option<String>,  // "DEPRECATED", "OBSOLETE", or empty
}
```

**New Methods on `Image`:**

```rust
impl Image {
    /// Check if image is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecated
            .as_ref()
            .and_then(|d| d.state.as_ref())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    /// Check if image was created within last 6 months
    pub fn is_recent(&self) -> bool {
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&self.creation_timestamp) {
            let six_months_ago = chrono::Utc::now() - chrono::Duration::days(180);
            created.with_timezone(&chrono::Utc) > six_months_ago
        } else {
            false
        }
    }

    /// Get human-readable family name for grouping
    pub fn family_group(&self) -> String {
        match self.family.as_deref() {
            Some("debian-13") => "Debian 13".to_string(),
            Some("debian-12") => "Debian 12".to_string(),
            Some("ubuntu-2404-lts") => "Ubuntu 24.04 LTS".to_string(),
            Some("ubuntu-2204-lts") => "Ubuntu 22.04 LTS".to_string(),
            Some(other) => other.replace('-', " ").to_uppercase(),
            None => self.name.clone(),
        }
    }

    /// Get display name with creation date for UI
    pub fn display_name(&self) -> String {
        let date = self.creation_timestamp
            .split('T')
            .next()
            .unwrap_or("");
        format!("{} ({})", self.family_group(), date)
    }
}
```

**New API Methods on `GcpRestClient`:**

```rust
impl GcpRestClient {
    /// List images from a public image project
    ///
    /// API: GET /projects/{project}/global/images
    pub fn list_images(&self, image_project: &str) -> Result<ImageList> {
        let url = format!(
            "{}/projects/{}/global/images",
            GCP_COMPUTE_API_BASE, image_project
        );
        let response = self.get(&url)?;
        Ok(response.into_json()?)
    }

    /// Get filtered list of recent Debian and Ubuntu images
    ///
    /// Filters:
    /// - OS: Debian or Ubuntu only
    /// - Architecture: X86_64
    /// - Age: Created within last 6 months
    /// - Status: Not deprecated
    pub fn list_debian_ubuntu_images(&self) -> Result<Vec<Image>> {
        let mut all_images = Vec::new();

        // Fetch Debian images
        match self.list_images("debian-cloud") {
            Ok(list) => all_images.extend(list.items),
            Err(e) => log::warn!("Failed to fetch Debian images: {}", e),
        }

        // Fetch Ubuntu images
        match self.list_images("ubuntu-os-cloud") {
            Ok(list) => all_images.extend(list.items),
            Err(e) => log::warn!("Failed to fetch Ubuntu images: {}", e),
        }

        // Apply filters
        let filtered: Vec<Image> = all_images
            .into_iter()
            .filter(|img| {
                let is_x86_64 = img
                    .architecture
                    .as_ref()
                    .map(|arch| arch.to_uppercase() == "X86_64")
                    .unwrap_or(false);

                let is_recent = img.is_recent();
                let not_deprecated = !img.is_deprecated();

                is_x86_64 && is_recent && not_deprecated
            })
            .collect();

        Ok(filtered)
    }
}
```

#### `mobile/src/ui_dlg/platform_gcp.rs`

**New Fields in `GcpWizard`:**

```rust
pub struct GcpWizard {
    // ... existing fields ...

    /// Selected source image (self_link URL)
    selected_image: String,

    /// Disk size in GB (user input as string)
    disk_size_gb: String,

    /// Available images (cached after successful load)
    #[cfg_attr(feature = "serde", serde(skip))]
    available_images: Vec<Image>,

    /// Image loading promise
    #[cfg_attr(feature = "serde", serde(skip))]
    image_promise: Option<Promise<Result<Vec<Image>, String>>>,

    /// Retry count for image loading (max 3)
    image_retry_count: u32,

    /// Whether to skip account/project steps
    skip_account_project_steps: bool,
}
```

**New Constructor:**

```rust
impl GcpWizard {
    /// Create wizard with platform context (skips account/project steps)
    pub fn with_platform_context(
        platform_name: String,
        project_id: String,
        oauth_result: OAuthResult,
    ) -> Self {
        Self {
            platform_name,
            selected_project_id: project_id,
            oauth_result: Some(oauth_result),
            skip_account_project_steps: true,
            state: WizardState::ConfigureServer,  // Start here
            ..Default::default()
        }
    }
}
```

**Modified Methods:**

```rust
impl GcpWizard {
    /// Modified: show() preserves state when context provided
    pub fn show(&mut self) {
        self.show = true;
        if !self.skip_account_project_steps {
            self.state = WizardState::ConnectAccount;
        }
        // else: keep state (ConfigureServer)
        self.progress_log.clear();
    }

    /// Modified: render_progress_indicator() hides skipped steps
    fn render_progress_indicator(&self, ui: &mut egui::Ui) {
        let steps = if self.skip_account_project_steps {
            vec![
                ("Configure", WizardState::ConfigureServer),
                ("Create", WizardState::CreatingServer),
                ("Complete", WizardState::Complete),
            ]
        } else {
            vec![
                ("Connect", WizardState::ConnectAccount),
                ("Project", WizardState::SelectProject),
                ("Configure", WizardState::ConfigureServer),
                ("Create", WizardState::CreatingServer),
                ("Complete", WizardState::Complete),
            ]
        };
        // ... render steps ...
    }

    /// Modified: render_configure_server() adds image/disk UI
    fn render_configure_server(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configure Server");
        ui.add_space(8.0);

        // Start image loading if not started
        if self.image_promise.is_none() && self.available_images.is_empty() {
            self.start_image_loading();
        }

        // Check promise result
        if let Some(promise) = &self.image_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(images) => {
                        self.available_images = images.clone();
                        self.image_promise = None;
                        // Auto-select latest image
                        if let Some(latest) = self.available_images.first() {
                            self.selected_image = latest.self_link.clone();
                        }
                    }
                    Err(_) if self.image_retry_count < 3 => {
                        // Retry with exponential backoff
                        self.image_retry_count += 1;
                        std::thread::sleep(std::time::Duration::from_secs(
                            2_u64.pow(self.image_retry_count)
                        ));
                        self.start_image_loading();
                    }
                    Err(_) => {
                        // Use fallback
                        self.available_images = get_fallback_images();
                        self.image_promise = None;
                    }
                }
            }
        }

        // Instance name
        ui.horizontal(|ui| {
            ui.label("Instance Name:");
            ui.text_edit_singleline(&mut self.instance_name);
        });
        // ... validation ...

        // Source Image (NEW)
        ui.horizontal(|ui| {
            ui.label("Source Image:");
            let selected_display = if self.image_promise.is_some() {
                if self.image_retry_count > 0 {
                    format!("⏳ Retrying... ({}/3)", self.image_retry_count)
                } else {
                    "⏳ Loading images...".to_string()
                }
            } else if let Some(img) = self.available_images
                .iter()
                .find(|img| img.self_link == self.selected_image)
            {
                img.display_name()
            } else {
                "Debian 13 (default)".to_string()
            };

            egui::ComboBox::from_id_salt("image_combo")
                .selected_text(&selected_display)
                .width(350.0)
                .show_ui(ui, |ui| {
                    // Group by family
                    let mut by_family: std::collections::HashMap<String, Vec<&Image>> =
                        std::collections::HashMap::new();
                    for img in &self.available_images {
                        by_family
                            .entry(img.family_group())
                            .or_default()
                            .push(img);
                    }

                    for (family, images) in by_family {
                        ui.label(egui::RichText::new(&family).strong());
                        for img in images {
                            ui.selectable_value(
                                &mut self.selected_image,
                                img.self_link.clone(),
                                format!("  {}", img.display_name()),
                            );
                        }
                        ui.add_space(4.0);
                    }
                });
        });

        // Disk Size (NEW)
        ui.horizontal(|ui| {
            ui.label("Disk Size (GB):");
            ui.add(egui::TextEdit::singleline(&mut self.disk_size_gb).desired_width(80.0));
            ui.colored_label(egui::Color32::GRAY, "Minimum: 10 GB");
        });

        // Validation
        if let Err(e) = validate_disk_size(&self.disk_size_gb) {
            ui.colored_label(egui::Color32::from_rgb(245, 101, 101), format!("⚠ {}", e));
        }

        // ... region, zone, machine type (unchanged) ...

        // Buttons
        ui.horizontal(|ui| {
            // Only show Back if came from full wizard
            if !self.skip_account_project_steps {
                if ui.button("← Back").clicked() {
                    self.state = WizardState::SelectProject;
                }
            }

            let can_create = !self.instance_name.is_empty()
                && validate_instance_name(&self.instance_name).is_ok()
                && validate_disk_size(&self.disk_size_gb).is_ok()
                && self.image_promise.is_none();  // Wait for loading

            ui.add_enabled_ui(can_create, |ui| {
                if ui.add(MaterialButton::filled("Create Server")).clicked() {
                    self.start_server_creation();
                }
            });

            if ui.button("Cancel").clicked() {
                self.hide();
            }
        });
    }

    /// New: Start async image loading
    fn start_image_loading(&mut self) {
        let access_token = self.oauth_result
            .as_ref()
            .map(|o| o.access_token.clone())
            .unwrap_or_default();

        self.image_promise = Some(Promise::spawn_thread("load_gcp_images", move || {
            let client = GcpRestClient::new(access_token);
            client.list_debian_ubuntu_images()
                .map_err(|e| e.to_string())
        }));
    }

    /// Modified: start_server_creation() uses selected image and disk
    fn start_server_creation(&mut self) {
        // ... existing setup ...

        let source_image = self.selected_image.clone();
        let disk_size_gb = self.disk_size_gb.clone();

        self.create_promise = Some(Promise::spawn_thread("gcp_create_vm", move || {
            // ... existing SSH key generation ...

            let mut instance_req = InstanceRequest::debian_micro(instance_name.clone(), zone.clone());
            
            // Apply custom image and disk size
            instance_req.disks[0].initialize_params.source_image = source_image;
            instance_req.disks[0].initialize_params.disk_size_gb = disk_size_gb;

            // ... rest unchanged ...
        }));
    }
}
```

**Helper Functions:**

```rust
/// Validate disk size (10 GB minimum, 65536 GB maximum)
fn validate_disk_size(input: &str) -> Result<u32, String> {
    let size = input.parse::<u32>()
        .map_err(|_| "Must be a valid number")?;
    
    if size < 10 {
        return Err("Minimum disk size is 10 GB".to_string());
    }
    
    if size > 65536 {
        return Err("Maximum disk size is 65536 GB".to_string());
    }
    
    Ok(size)
}

/// Get fallback images when API call fails
fn get_fallback_images() -> Vec<Image> {
    vec![
        Image {
            name: "debian-13-latest".to_string(),
            description: Some("Debian 13 (Bookworm) - latest".to_string()),
            self_link: "projects/debian-cloud/global/images/family/debian-13".to_string(),
            creation_timestamp: chrono::Utc::now().to_rfc3339(),
            architecture: Some("X86_64".to_string()),
            family: Some("debian-13".to_string()),
            deprecated: None,
        },
        Image {
            name: "ubuntu-2404-lts-latest".to_string(),
            description: Some("Ubuntu 24.04 LTS - latest".to_string()),
            self_link: "projects/ubuntu-os-cloud/global/images/family/ubuntu-2404-lts".to_string(),
            creation_timestamp: chrono::Utc::now().to_rfc3339(),
            architecture: Some("X86_64".to_string()),
            family: Some("ubuntu-2404-lts".to_string()),
            deprecated: None,
        },
    ]
}

/// Validate instance name (existing function, no changes)
fn validate_instance_name(name: &str) -> Result<(), String> {
    // ... existing validation logic ...
}
```

## Data Flow

### Image Loading Flow

```
User clicks "Add VM" in Platform tab
    ↓
GcpWizard::with_platform_context(platform, project, oauth)
    ↓
wizard.show() → state = ConfigureServer
    ↓
render_configure_server() first render
    ↓
start_image_loading() spawns Promise
    │
    ├─→ GcpRestClient.list_debian_ubuntu_images()
    │       │
    │       ├─→ Fetch from debian-cloud
    │       ├─→ Fetch from ubuntu-os-cloud
    │       ├─→ Filter: architecture == "X86_64"
    │       ├─→ Filter: created within 6 months
    │       └─→ Filter: not deprecated
    │
    ├─→ SUCCESS: available_images populated, auto-select latest
    │
    └─→ FAILURE:
            If retry_count < 3:
                Wait 2^retry_count seconds
                Retry start_image_loading()
            Else:
                Use get_fallback_images()
```

### VM Creation Flow

```
User fills: name, image, disk, region, zone, machine_type
    ↓
Validation passes (disk ≥ 10GB, name valid, image loaded)
    ↓
User clicks "Create Server"
    ↓
start_server_creation()
    │
    ├─→ Apply selected_image to InstanceRequest.disks[0].initialize_params.source_image
    ├─→ Apply disk_size_gb to InstanceRequest.disks[0].initialize_params.disk_size_gb
    │
    └─→ Spawn Promise (existing VM creation flow unchanged)
```

### State Transitions

```
[Platform Tab: Add VM clicked]
    ↓
ConfigureServer (skip_account_project_steps = true)
    ↓
Image loading: "⏳ Loading images..."
    ↓ (on success)
Dropdown populated with grouped images
    ↓
User configures and clicks Create
    ↓
CreatingServer → Complete
```

## Error Handling & Retry Logic

### Retry Strategy

**Exponential Backoff:**
- Attempt 1: Immediate
- Attempt 2: Wait 2 seconds
- Attempt 3: Wait 4 seconds
- Max attempts: 3

```rust
const MAX_IMAGE_LOAD_RETRIES: u32 = 3;

fn calculate_retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(2_u64.pow(attempt))
}
```

### Error Scenarios

| Error Type | User Message | Retry | Fallback |
|------------|--------------|-------|----------|
| **Network timeout** | "⏳ Retrying... (N/3)" | Yes | After 3 attempts |
| **401 Unauthorized** | "⚠ Token expired" | No | Immediate |
| **403 Forbidden** | "⚠ API access denied" | No | Immediate |
| **Compute API disabled** | "ℹ Using default images" | No | Immediate |
| **Empty result** | "ℹ No recent images found" | No | Immediate |
| **All retries exhausted** | "⚠ Could not load images (using defaults)" | No | Use fallback |

### Fallback Behavior

When API calls fail or exhaust retries, provide minimal curated list:
- `projects/debian-cloud/global/images/family/debian-13` (latest Debian 13)
- `projects/ubuntu-os-cloud/global/images/family/ubuntu-2404-lts` (latest Ubuntu 24.04 LTS)

These use GCP's `/family/` URL pattern, which always resolves to the newest image in that family.

### Disk Size Validation

```rust
fn validate_disk_size(input: &str) -> Result<u32, String> {
    let size = input.parse::<u32>()
        .map_err(|_| "Must be a valid number")?;
    
    if size < 10 {
        return Err("Minimum disk size is 10 GB".to_string());
    }
    
    if size > 65536 {  // GCP limit
        return Err("Maximum disk size is 65536 GB".to_string());
    }
    
    Ok(size)
}
```

## Testing Strategy (TDD)

### Unit Tests - `mobile/src/api/gcp/compute.rs`

All tests in `#[cfg(test)]` module at end of file.

#### 1. Image Filtering Logic

```rust
#[test]
fn test_image_is_recent() {
    use chrono::{Utc, Duration};
    
    // Recent image (3 months old)
    let recent = Image {
        creation_timestamp: (Utc::now() - Duration::days(90)).to_rfc3339(),
        ..Default::default()
    };
    assert!(recent.is_recent());
    
    // Old image (7 months old)
    let old = Image {
        creation_timestamp: (Utc::now() - Duration::days(210)).to_rfc3339(),
        ..Default::default()
    };
    assert!(!old.is_recent());
}

#[test]
fn test_image_is_deprecated() {
    // Active image
    let active = Image {
        deprecated: None,
        ..Default::default()
    };
    assert!(!active.is_deprecated());
    
    // Deprecated image
    let deprecated = Image {
        deprecated: Some(DeprecatedStatus {
            state: Some("DEPRECATED".to_string()),
        }),
        ..Default::default()
    };
    assert!(deprecated.is_deprecated());
    
    // Edge case: empty state string
    let edge = Image {
        deprecated: Some(DeprecatedStatus {
            state: Some("".to_string()),
        }),
        ..Default::default()
    };
    assert!(!edge.is_deprecated());
}

#[test]
fn test_image_architecture_filter() {
    let x86 = Image {
        architecture: Some("X86_64".to_string()),
        ..Default::default()
    };
    let arm = Image {
        architecture: Some("ARM64".to_string()),
        ..Default::default()
    };
    let none = Image {
        architecture: None,
        ..Default::default()
    };
    
    assert!(matches_x86_64(&x86));
    assert!(!matches_x86_64(&arm));
    assert!(!matches_x86_64(&none));
}
```

#### 2. Display Formatting

```rust
#[test]
fn test_image_display_name() {
    let img = Image {
        name: "debian-13-bookworm-v20260615".to_string(),
        family: Some("debian-13".to_string()),
        creation_timestamp: "2026-06-15T10:00:00.000Z".to_string(),
        ..Default::default()
    };
    
    assert_eq!(img.display_name(), "Debian 13 (2026-06-15)");
}

#[test]
fn test_image_family_group() {
    let debian = Image {
        family: Some("debian-13".to_string()),
        ..Default::default()
    };
    assert_eq!(debian.family_group(), "Debian 13");
    
    let ubuntu = Image {
        family: Some("ubuntu-2404-lts".to_string()),
        ..Default::default()
    };
    assert_eq!(ubuntu.family_group(), "Ubuntu 24.04 LTS");
    
    let unknown = Image {
        family: Some("custom-os-v1".to_string()),
        ..Default::default()
    };
    assert_eq!(unknown.family_group(), "CUSTOM OS V1");
}
```

#### 3. Validation Functions

```rust
#[test]
fn test_validate_disk_size() {
    // Valid sizes
    assert!(validate_disk_size("10").is_ok());
    assert!(validate_disk_size("100").is_ok());
    assert!(validate_disk_size("65536").is_ok());
    
    // Invalid: too small
    assert_eq!(
        validate_disk_size("9").unwrap_err(),
        "Minimum disk size is 10 GB"
    );
    
    // Invalid: too large
    assert_eq!(
        validate_disk_size("65537").unwrap_err(),
        "Maximum disk size is 65536 GB"
    );
    
    // Invalid: not a number
    assert!(validate_disk_size("abc").is_err());
    assert!(validate_disk_size("").is_err());
    assert!(validate_disk_size("-5").is_err());
}

#[test]
fn test_validate_instance_name() {
    // Valid names
    assert!(validate_instance_name("my-server").is_ok());
    assert!(validate_instance_name("server123").is_ok());
    assert!(validate_instance_name("a").is_ok());
    
    // Invalid: starts with number
    assert!(validate_instance_name("123server").is_err());
    
    // Invalid: contains uppercase
    assert!(validate_instance_name("MyServer").is_err());
    
    // Invalid: too long (>63 chars)
    assert!(validate_instance_name(&"a".repeat(64)).is_err());
    
    // Invalid: empty
    assert!(validate_instance_name("").is_err());
}
```

#### 4. JSON Parsing

```rust
#[test]
fn test_parse_image_list_response() {
    let json = r#"{
        "items": [
            {
                "name": "debian-13-bookworm-v20260615",
                "selfLink": "projects/debian-cloud/global/images/debian-13-bookworm-v20260615",
                "creationTimestamp": "2026-06-15T10:00:00.000Z",
                "architecture": "X86_64",
                "family": "debian-13"
            }
        ]
    }"#;
    
    let list: ImageList = serde_json::from_str(json).unwrap();
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].name, "debian-13-bookworm-v20260615");
    assert_eq!(list.items[0].architecture.as_deref(), Some("X86_64"));
}

#[test]
fn test_parse_deprecated_image() {
    let json = r#"{
        "name": "old-image",
        "selfLink": "projects/debian-cloud/global/images/old-image",
        "creationTimestamp": "2020-01-01T00:00:00.000Z",
        "architecture": "X86_64",
        "deprecated": {
            "state": "DEPRECATED"
        }
    }"#;
    
    let img: Image = serde_json::from_str(json).unwrap();
    assert!(img.is_deprecated());
}
```

### Test Execution

Run tests with:
```bash
cargo test --lib api::gcp::compute::tests
```

### Coverage Target

- **Image filtering:** 100% (is_recent, is_deprecated, architecture check)
- **Display formatting:** 100% (display_name, family_group)
- **Validation:** 100% (disk size, instance name)
- **JSON parsing:** Sample coverage (common cases)

## UI/UX Specification

### Dialog Flow Comparison

**Before (from "Add Platform"):**
```
[1. Connect Account] → [2. Select Project] → [3. Configure] → [4. Create] → [5. Complete]
```

**After (from "Add VM" on platform row):**
```
[1. Configure] → [2. Create] → [3. Complete]
```

### ConfigureServer Step Layout

```
┌──────────────────────────────────────────────────────────┐
│ Configure Server                                          │
├──────────────────────────────────────────────────────────┤
│                                                           │
│ Instance Name: [dure-server____________________]          │
│ ✓ Valid name                                              │
│                                                           │
│ Source Image:  [Debian 13 (2026-06-15)         ▼]        │
│   (dropdown grouped by family, sorted newest first)       │
│                                                           │
│ Disk Size (GB): [10___] Minimum: 10 GB                   │
│                                                           │
│ Region:        [Iowa, USA (us-central1)        ▼]        │
│ Zone:          [us-central1-a                  ▼]        │
│ Machine Type:  [e2-micro - Shared-core machine ▼]        │
│                                                           │
│ [Cancel] [Create Server]                                  │
└──────────────────────────────────────────────────────────┘
```

### Loading States

**Initial load:**
```
Source Image: [⏳ Loading images...              ▼]
```

**Retry state:**
```
Source Image: [⏳ Retrying... (2/3)             ▼]
```

**Fallback state:**
```
Source Image: [Debian 13 (default)             ▼]
⚠ Could not load images (using defaults)
```

**Loaded state (grouped dropdown):**
```
Source Image: [Debian 13 (2026-06-15)          ▼]

Dropdown contents:
  Debian 13
    Debian 13 (2026-06-15)
    Debian 13 (2026-06-01)
    Debian 13 (2026-05-20)
  
  Ubuntu 24.04 LTS
    Ubuntu 24.04 LTS (2026-06-10)
    Ubuntu 24.04 LTS (2026-05-25)
  
  Ubuntu 22.04 LTS
    Ubuntu 22.04 LTS (2026-06-08)
```

### Progress Indicator

**Full wizard:**
```
● Connect → Project → Configure → Create → Complete
```

**Shortened wizard:**
```
● Configure → Create → Complete
```

### Validation Messages

**Disk size:**
- Input "5" → `⚠ Minimum disk size is 10 GB`
- Input "abc" → `⚠ Must be a valid number`
- Input "100" → No message (valid)

**Image selection:**
- Loading → Spinner, button disabled
- Loaded → Dropdown enabled
- Failed → Fallback list, warning message

### Accessibility

- Loading states have text labels for screen readers
- Retry count visible: "Retrying (2 of 3)"
- Validation errors announced on change
- Keyboard navigation through grouped dropdown

## Implementation Notes

### Existing Patterns to Follow

**Async with Promise:**
```rust
// Existing pattern in platform_gcp.rs
self.oauth_promise = Some(Promise::spawn_thread("gcp_oauth", move || {
    handler.run_oauth_flow().map_err(|e| e.to_string())
}));

// New pattern for images
self.image_promise = Some(Promise::spawn_thread("load_gcp_images", move || {
    client.list_debian_ubuntu_images().map_err(|e| e.to_string())
}));
```

**Promise result checking:**
```rust
// Check in render loop
if let Some(promise) = &self.image_promise {
    if let Some(result) = promise.ready() {
        match result {
            Ok(data) => { /* success */ },
            Err(e) => { /* handle error */ },
        }
    }
}
```

### Dependencies

**Already in Cargo.toml:**
- `poll_promise = "0.3"` - Async promise handling
- `chrono = "0.4"` - Date/time parsing for age filter
- `serde = { version = "1.0", features = ["derive"] }` - JSON deserialization
- `egui = "0.33"` - UI framework

**No new dependencies needed.**

### File Changes Summary

**Modified:**
- `mobile/src/api/gcp/compute.rs` - Add image types and API methods
- `mobile/src/ui_dlg/platform_gcp.rs` - Add image/disk UI, new constructor

**Test additions:**
- `mobile/src/api/gcp/compute.rs` - Add `#[cfg(test)]` module with unit tests

**No files deleted.**

## Migration Path

### Backward Compatibility

**Existing wizard flow preserved:**
- Opening from "Add Platform" button still uses full 5-step flow
- Only "Add VM" button uses shortened flow

**Existing VM creation unchanged:**
- Users who don't change image/disk get same defaults
- No breaking changes to API calls

### Rollout Plan

1. **Phase 1:** Implement API layer (compute.rs) with tests - no UI changes yet
2. **Phase 2:** Add new constructor and skip logic - coexists with old flow
3. **Phase 3:** Add image/disk UI - visible but uses defaults initially
4. **Phase 4:** Enable async loading - full feature live

Each phase is independently testable and deployable.

## Success Metrics

**Functional:**
- ✅ Image list loads successfully from GCP API
- ✅ Filtering reduces list to recent, x86_64, non-deprecated images
- ✅ Auto-retry succeeds on transient network errors
- ✅ Fallback images work when API unavailable
- ✅ VM creation succeeds with custom image and disk size

**UX:**
- ✅ Users skip 2 unnecessary steps when context known
- ✅ Image selection shows creation dates for informed choice
- ✅ Loading state visible during async fetch
- ✅ Validation prevents invalid disk sizes

**Quality:**
- ✅ All unit tests pass (filtering, validation, parsing)
- ✅ No regressions in existing wizard flows
- ✅ Code follows existing patterns (Promise, layered architecture)

## Future Enhancements (Out of Scope)

- ARM64 architecture support
- Windows/CentOS image families
- Custom image upload
- Global image cache across app sessions
- Image preview/description tooltips
- Disk type selection (SSD vs Standard)

---

**Design approved by:** User  
**Ready for implementation:** Yes  
**Next step:** Invoke writing-plans skill to create TDD implementation plan
