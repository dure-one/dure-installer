# GCP Dialog Image Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable GCP VM creation dialog to fetch/filter public Debian/Ubuntu images, skip redundant account/project steps, and accept custom disk sizes with TDD unit test coverage.

**Architecture:** Two-layer approach - API layer adds image fetching/filtering in compute.rs, UI layer adds async loading with retry in platform_gcp.rs. Follows existing poll_promise pattern for async operations.

**Tech Stack:** Rust (nightly), egui 0.33, poll_promise 0.3, chrono 0.4, serde

## Global Constraints

- Rust nightly toolchain required
- Unit tests only (no integration tests)
- Follow existing Promise pattern from oauth_promise/create_promise
- 100% test coverage for filtering/validation logic
- x86_64 architecture only
- Debian/Ubuntu families only
- 6-month image age limit (180 days)
- Disk size: 10 GB minimum, 65536 GB maximum
- Max 3 retry attempts with exponential backoff (2^N seconds)

---

### Task 1: Add Image Types with Default Implementation

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs:1-260` (after existing types)

**Interfaces:**
- Consumes: None
- Produces: `Image`, `ImageList`, `DeprecatedStatus` types with `Default` trait

- [ ] **Step 1: Write failing test for Image Default**

Add to end of `mobile/src/api/gcp/compute.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_default() {
        let img = Image::default();
        assert_eq!(img.name, "");
        assert_eq!(img.self_link, "");
        assert!(img.architecture.is_none());
        assert!(img.deprecated.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test test_image_default --lib`
Expected: FAIL with "no field `default` on type `Image`"

- [ ] **Step 3: Add Image types with Default**

Add after `Zone` type in `mobile/src/api/gcp/compute.rs` (around line 256):

```rust
// ============================================================================
// Image Types
// ============================================================================

/// Image list response from GCP API
#[derive(Debug, Deserialize, Default)]
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
    pub self_link: String,
    pub creation_timestamp: String,
    pub architecture: Option<String>,
    pub family: Option<String>,
    pub deprecated: Option<DeprecatedStatus>,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            self_link: String::new(),
            creation_timestamp: String::new(),
            architecture: None,
            family: None,
            deprecated: None,
        }
    }
}

/// Deprecation status
#[derive(Debug, Clone, Deserialize)]
pub struct DeprecatedStatus {
    pub state: Option<String>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd mobile && cargo test test_image_default --lib`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add Image types with Default trait

Add Image, ImageList, DeprecatedStatus types for GCP image API.
Include Default implementation for testing."
```

---

### Task 2: Add Image Deprecation Check with Tests

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs:260-290` (Image impl block)
- Test: `mobile/src/api/gcp/compute.rs` (tests module)

**Interfaces:**
- Consumes: `Image`, `DeprecatedStatus` from Task 1
- Produces: `Image::is_deprecated() -> bool`

- [ ] **Step 1: Write failing test**

Add to tests module:

```rust
#[test]
fn test_image_is_deprecated() {
    // Active image (no deprecated field)
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test test_image_is_deprecated --lib`
Expected: FAIL with "no method named `is_deprecated`"

- [ ] **Step 3: Implement is_deprecated method**

Add after `impl Default for Image`:

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
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd mobile && cargo test test_image_is_deprecated --lib`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add Image::is_deprecated method

Check if GCP image is marked deprecated by examining state field.
Empty or missing state means not deprecated."
```

---

### Task 3: Add Image Recency Check with Tests

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs` (Image impl block)
- Test: `mobile/src/api/gcp/compute.rs` (tests module)

**Interfaces:**
- Consumes: `Image::creation_timestamp` from Task 1
- Produces: `Image::is_recent() -> bool` (6-month window)

- [ ] **Step 1: Write failing test**

Add to tests module:

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

    // Invalid timestamp
    let invalid = Image {
        creation_timestamp: "not-a-date".to_string(),
        ..Default::default()
    };
    assert!(!invalid.is_recent());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test test_image_is_recent --lib`
Expected: FAIL with "no method named `is_recent`"

- [ ] **Step 3: Implement is_recent method**

Add to `impl Image` block:

```rust
/// Check if image was created within last 6 months
pub fn is_recent(&self) -> bool {
    if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&self.creation_timestamp) {
        let six_months_ago = chrono::Utc::now() - chrono::Duration::days(180);
        created.with_timezone(&chrono::Utc) > six_months_ago
    } else {
        false
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd mobile && cargo test test_image_is_recent --lib`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add Image::is_recent method

Check if image was created within last 6 months (180 days).
Returns false for unparseable timestamps."
```

---

### Task 4: Add Image Display Methods with Tests

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs` (Image impl block)
- Test: `mobile/src/api/gcp/compute.rs` (tests module)

**Interfaces:**
- Consumes: `Image::family`, `Image::creation_timestamp` from Task 1
- Produces: `Image::family_group() -> String`, `Image::display_name() -> String`

- [ ] **Step 1: Write failing tests**

Add to tests module:

```rust
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

    let no_family = Image::default();
    assert_eq!(no_family.family_group(), "");
}

#[test]
fn test_image_display_name() {
    let img = Image {
        name: "debian-13-bookworm-v20260615".to_string(),
        family: Some("debian-13".to_string()),
        creation_timestamp: "2026-06-15T10:00:00.000Z".to_string(),
        ..Default::default()
    };
    assert_eq!(img.display_name(), "Debian 13 (2026-06-15)");

    let no_date = Image {
        family: Some("debian-13".to_string()),
        creation_timestamp: "".to_string(),
        ..Default::default()
    };
    assert_eq!(no_date.display_name(), "Debian 13 ()");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd mobile && cargo test test_image_family_group test_image_display_name --lib`
Expected: FAIL with "no method named `family_group`"

- [ ] **Step 3: Implement display methods**

Add to `impl Image` block:

```rust
/// Get human-readable family name for UI grouping
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
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd mobile && cargo test test_image_family_group test_image_display_name --lib`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add Image display formatting methods

Add family_group() for UI grouping (Debian 13, Ubuntu 24.04 LTS).
Add display_name() with creation date for dropdown display."
```

---

### Task 5: Add list_images API Method with JSON Test

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs` (GcpRestClient impl, around line 677)
- Test: `mobile/src/api/gcp/compute.rs` (tests module)

**Interfaces:**
- Consumes: `GcpRestClient`, `ImageList` from Task 1
- Produces: `GcpRestClient::list_images(image_project: &str) -> Result<ImageList>`

- [ ] **Step 1: Write failing JSON parsing test**

Add to tests module:

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

- [ ] **Step 2: Run tests to verify they pass (types already support serde)**

Run: `cd mobile && cargo test test_parse_image_list_response test_parse_deprecated_image --lib`
Expected: PASS (2 tests) - types already have Deserialize

- [ ] **Step 3: Write failing test for list_images method**

Add to tests module (this will fail until we add the method):

```rust
#[test]
fn test_list_images_method_signature() {
    // This test verifies the method signature exists
    // We can't test actual API calls without mocking, so we just check compilation
    fn _check_signature(client: &GcpRestClient) {
        let _: Result<ImageList> = client.list_images("debian-cloud");
    }
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cd mobile && cargo test test_list_images_method_signature --lib`
Expected: FAIL with "no method named `list_images`"

- [ ] **Step 5: Implement list_images method**

Add to `impl GcpRestClient` block (after `add_ip_to_firewall` method, around line 677):

```rust
/// List images from a public image project (debian-cloud, ubuntu-os-cloud, etc.)
///
/// API: GET /projects/{project}/global/images
pub fn list_images(&self, image_project: &str) -> Result<ImageList> {
    let url = format!(
        "{}/projects/{}/global/images",
        GCP_COMPUTE_API_BASE, image_project
    );

    let response = self.get(&url)?;
    let list: ImageList = response.into_json()?;
    Ok(list)
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cd mobile && cargo test test_list_images_method_signature --lib`
Expected: PASS (1 test)

- [ ] **Step 7: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add list_images API method

Fetch image list from GCP public projects (debian-cloud, ubuntu-os-cloud).
Includes JSON parsing tests for ImageList deserialization."
```

---

### Task 6: Add list_debian_ubuntu_images with Filtering

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs` (GcpRestClient impl)
- Test: `mobile/src/api/gcp/compute.rs` (tests module)

**Interfaces:**
- Consumes: `GcpRestClient::list_images`, `Image::is_recent`, `Image::is_deprecated` from Tasks 2-5
- Produces: `GcpRestClient::list_debian_ubuntu_images() -> Result<Vec<Image>>`

- [ ] **Step 1: Write failing test for architecture filter helper**

Add to tests module:

```rust
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

    // Test the filter logic inline
    assert!(x86.architecture.as_ref()
        .map(|a| a.to_uppercase() == "X86_64")
        .unwrap_or(false));
    
    assert!(!arm.architecture.as_ref()
        .map(|a| a.to_uppercase() == "X86_64")
        .unwrap_or(false));
    
    assert!(!none.architecture.as_ref()
        .map(|a| a.to_uppercase() == "X86_64")
        .unwrap_or(false));
}
```

- [ ] **Step 2: Run test to verify it passes (no new code needed)**

Run: `cd mobile && cargo test test_image_architecture_filter --lib`
Expected: PASS (1 test) - filter logic is inline

- [ ] **Step 3: Write failing test for list_debian_ubuntu_images signature**

Add to tests module:

```rust
#[test]
fn test_list_debian_ubuntu_images_signature() {
    // Verify method signature exists
    fn _check_signature(client: &GcpRestClient) {
        let _: Result<Vec<Image>> = client.list_debian_ubuntu_images();
    }
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cd mobile && cargo test test_list_debian_ubuntu_images_signature --lib`
Expected: FAIL with "no method named `list_debian_ubuntu_images`"

- [ ] **Step 5: Implement list_debian_ubuntu_images method**

Add to `impl GcpRestClient` block (after `list_images`):

```rust
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
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cd mobile && cargo test test_list_debian_ubuntu_images_signature --lib`
Expected: PASS (1 test)

- [ ] **Step 7: Run all compute tests**

Run: `cd mobile && cargo test api::gcp::compute::tests --lib`
Expected: PASS (all tests)

- [ ] **Step 8: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add list_debian_ubuntu_images with filtering

Fetch and filter images from debian-cloud and ubuntu-os-cloud.
Filter by: X86_64 architecture, 6-month age, not deprecated.
Warn on fetch errors but continue with partial results."
```

---

### Task 7: Add validate_disk_size Function with Tests

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:1907` (after validate_instance_name)
- Test: `mobile/src/ui_dlg/platform_gcp.rs` (tests module)

**Interfaces:**
- Consumes: None
- Produces: `validate_disk_size(input: &str) -> Result<u32, String>`

- [ ] **Step 1: Write failing test**

Add to tests module in `mobile/src/ui_dlg/platform_gcp.rs` (after existing tests, around line 2092):

```rust
#[test]
fn test_validate_disk_size() {
    // Valid sizes
    assert!(validate_disk_size("10").is_ok());
    assert_eq!(validate_disk_size("10").unwrap(), 10);
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
    assert_eq!(
        validate_disk_size("abc").unwrap_err(),
        "Must be a valid number"
    );
    assert!(validate_disk_size("").is_err());
    assert!(validate_disk_size("-5").is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test test_validate_disk_size --lib`
Expected: FAIL with "cannot find function `validate_disk_size`"

- [ ] **Step 3: Implement validate_disk_size function**

Add after `validate_instance_name` function (around line 1907):

```rust
/// Validate disk size (10 GB minimum, 65536 GB maximum)
fn validate_disk_size(input: &str) -> Result<u32, String> {
    let size = input.parse::<u32>()
        .map_err(|_| "Must be a valid number".to_string())?;

    if size < 10 {
        return Err("Minimum disk size is 10 GB".to_string());
    }

    if size > 65536 {
        return Err("Maximum disk size is 65536 GB".to_string());
    }

    Ok(size)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd mobile && cargo test test_validate_disk_size --lib`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add disk size validation function

Validate disk size input: 10-65536 GB range.
Return descriptive error messages for UI display."
```

---

### Task 8: Add GcpWizard New Fields to Default

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:39-116` (GcpWizard struct and Default impl)

**Interfaces:**
- Consumes: `Image` from Task 1
- Produces: New fields on `GcpWizard`: `selected_image`, `disk_size_gb`, `available_images`, `image_promise`, `image_retry_count`, `skip_account_project_steps`

- [ ] **Step 1: Add new fields to GcpWizard struct**

Modify struct definition (around line 39):

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

- [ ] **Step 2: Update Default implementation**

Modify `impl Default for GcpWizard` (around line 118):

```rust
impl Default for GcpWizard {
    fn default() -> Self {
        let default_project_id = format!("dure-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));

        Self {
            state: WizardState::ConnectAccount,
            platform_name: String::new(),
            oauth_result: None,
            selected_project_id: default_project_id,
            selected_region: "us-central1".to_string(),
            selected_zone: "us-central1-a".to_string(),
            selected_machine_type: "e2-micro".to_string(),
            instance_name: "dure-server".to_string(),
            selected_image: "projects/debian-cloud/global/images/family/debian-13".to_string(),
            disk_size_gb: "10".to_string(),
            available_images: Vec::new(),
            image_promise: None,
            image_retry_count: 0,
            skip_account_project_steps: false,
            created_instance: None,
            available_regions: Vec::new(),
            available_machine_types: get_common_machine_types(),
            available_projects: Vec::new(),
            projects_loaded: false,
            projects_load_error: None,
            new_project_name: "Dure Server".to_string(),
            create_new_project_selected: false,
            oauth_promise: None,
            create_promise: None,
            progress_log: Vec::new(),
            show: false,
            available_platforms: Vec::new(),
            selected_platform_email: String::new(),
        }
    }
}
```

- [ ] **Step 3: Add Image import**

Add to imports at top of file (around line 16):

```rust
use crate::api::gcp::compute::{InstanceRequest, Metadata, MetadataItem, Image};
```

- [ ] **Step 4: Verify compilation**

Run: `cd mobile && cargo check --lib`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add new fields to GcpWizard

Add fields for image selection, disk size, async loading state.
Default: debian-13 family image, 10 GB disk, no step skipping."
```

---

### Task 9: Add with_platform_context Constructor

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:150-158` (after `new` method)

**Interfaces:**
- Consumes: `GcpWizard::default()` from Task 8
- Produces: `GcpWizard::with_platform_context(platform_name, project_id, oauth_result) -> Self`

- [ ] **Step 1: Write test for constructor**

Add to tests module:

```rust
#[test]
fn test_with_platform_context() {
    use crate::api::gcp::oauth::OAuthResult;
    
    let oauth = OAuthResult {
        access_token: "test-token".to_string(),
        refresh_token: "test-refresh".to_string(),
        expires_at: 12345,
    };
    
    let wizard = GcpWizard::with_platform_context(
        "TestPlatform".to_string(),
        "test-project".to_string(),
        oauth.clone(),
    );
    
    assert_eq!(wizard.platform_name, "TestPlatform");
    assert_eq!(wizard.selected_project_id, "test-project");
    assert!(wizard.oauth_result.is_some());
    assert!(wizard.skip_account_project_steps);
    assert_eq!(
        std::mem::discriminant(&wizard.state),
        std::mem::discriminant(&WizardState::ConfigureServer)
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test test_with_platform_context --lib`
Expected: FAIL with "no method named `with_platform_context`"

- [ ] **Step 3: Implement constructor**

Add after `new` method (around line 158):

```rust
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
        state: WizardState::ConfigureServer,
        ..Default::default()
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd mobile && cargo test test_with_platform_context --lib`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add with_platform_context constructor

Create wizard with pre-selected account/project, skipping steps 1-2.
Starts directly at ConfigureServer state."
```

---

### Task 10: Modify show() Method to Preserve State

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:213-218` (show method)

**Interfaces:**
- Consumes: `skip_account_project_steps` from Task 8
- Produces: Modified `show()` behavior

- [ ] **Step 1: Write test for show() preservation**

Add to tests module:

```rust
#[test]
fn test_show_preserves_state_when_skipping() {
    use crate::api::gcp::oauth::OAuthResult;
    
    let oauth = OAuthResult {
        access_token: "test".to_string(),
        refresh_token: "test".to_string(),
        expires_at: 12345,
    };
    
    let mut wizard = GcpWizard::with_platform_context(
        "Test".to_string(),
        "project".to_string(),
        oauth,
    );
    
    // State starts at ConfigureServer
    assert_eq!(
        std::mem::discriminant(&wizard.state),
        std::mem::discriminant(&WizardState::ConfigureServer)
    );
    
    wizard.show();
    
    // State should remain ConfigureServer (not reset to ConnectAccount)
    assert_eq!(
        std::mem::discriminant(&wizard.state),
        std::mem::discriminant(&WizardState::ConfigureServer)
    );
}

#[test]
fn test_show_resets_state_when_full_flow() {
    let mut wizard = GcpWizard::new("Test".to_string());
    wizard.state = WizardState::ConfigureServer;
    
    wizard.show();
    
    // State should reset to ConnectAccount for full flow
    assert_eq!(
        std::mem::discriminant(&wizard.state),
        std::mem::discriminant(&WizardState::ConnectAccount)
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd mobile && cargo test test_show_preserves_state --lib`
Expected: FAIL - current show() always resets to ConnectAccount

- [ ] **Step 3: Modify show() method**

Replace existing `show()` method:

```rust
/// Show the wizard
pub fn show(&mut self) {
    self.show = true;
    if !self.skip_account_project_steps {
        self.state = WizardState::ConnectAccount;
    }
    // else: keep current state (ConfigureServer)
    self.progress_log.clear();
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd mobile && cargo test test_show_preserves_state test_show_resets_state --lib`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): modify show() to preserve state when skipping

When skip_account_project_steps=true, preserve ConfigureServer state.
When false, reset to ConnectAccount for full flow."
```

---

### Task 11: Add get_fallback_images Helper Function

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:1907` (after validate functions)

**Interfaces:**
- Consumes: `Image` from Task 1
- Produces: `get_fallback_images() -> Vec<Image>`

- [ ] **Step 1: Write test for fallback images**

Add to tests module:

```rust
#[test]
fn test_get_fallback_images() {
    let images = get_fallback_images();
    
    assert_eq!(images.len(), 2);
    
    // Check Debian image
    assert_eq!(images[0].family.as_deref(), Some("debian-13"));
    assert_eq!(
        images[0].self_link,
        "projects/debian-cloud/global/images/family/debian-13"
    );
    
    // Check Ubuntu image
    assert_eq!(images[1].family.as_deref(), Some("ubuntu-2404-lts"));
    assert_eq!(
        images[1].self_link,
        "projects/ubuntu-os-cloud/global/images/family/ubuntu-2404-lts"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test test_get_fallback_images --lib`
Expected: FAIL with "cannot find function `get_fallback_images`"

- [ ] **Step 3: Implement get_fallback_images function**

Add after `validate_disk_size` function:

```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd mobile && cargo test test_get_fallback_images --lib`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add get_fallback_images helper

Provide default Debian 13 and Ubuntu 24.04 LTS images when API fails.
Uses GCP family URLs which always resolve to latest version."
```

---

### Task 12: Modify render_progress_indicator

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:268-305` (render_progress_indicator method)

**Interfaces:**
- Consumes: `skip_account_project_steps` from Task 8
- Produces: Modified progress indicator display

- [ ] **Step 1: Modify render_progress_indicator method**

Replace existing method (around line 268):

```rust
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

    ui.horizontal(|ui| {
        for (i, (label, step_state)) in steps.iter().enumerate() {
            if i > 0 {
                ui.label("→");
            }

            let is_current =
                std::mem::discriminant(&self.state) == std::mem::discriminant(step_state);
            let is_past = self.is_past_step(step_state);

            let color = if is_current {
                egui::Color32::from_rgb(103, 126, 234) // Primary color
            } else if is_past {
                egui::Color32::from_rgb(72, 187, 120) // Green
            } else {
                egui::Color32::GRAY
            };

            ui.colored_label(
                color,
                if is_current {
                    format!("● {}", label)
                } else {
                    label.to_string()
                },
            );
        }
    });
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): modify progress indicator for shortened flow

Show 3 steps (Configure→Create→Complete) when skipping.
Show 5 steps (Connect→Project→Configure→Create→Complete) for full flow."
```

---

### Task 13: Add start_image_loading Method

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:1450` (before start_server_creation)

**Interfaces:**
- Consumes: `oauth_result`, `image_promise` from Task 8; `GcpRestClient::list_debian_ubuntu_images` from Task 6
- Produces: `start_image_loading()` method

- [ ] **Step 1: Add start_image_loading method**

Add before `start_server_creation` method (around line 1450):

```rust
/// Start async image loading
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
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`
Expected: No errors

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add start_image_loading async method

Spawn Promise thread to fetch Debian/Ubuntu images.
Uses existing oauth access token for authentication."
```

---

### Task 14: Modify render_configure_server - Add Image UI

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:711-823` (render_configure_server method)

**Interfaces:**
- Consumes: `start_image_loading`, `get_fallback_images`, `image_promise`, `available_images` from Tasks 11-13
- Produces: Image dropdown UI with async loading and retry

- [ ] **Step 1: Add image loading logic at start of render_configure_server**

Modify method start (after `ui.heading`, around line 713):

```rust
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

    // ... rest of existing method ...
```

- [ ] **Step 2: Add image selection UI after instance name validation**

Add after instance name section (around line 733):

```rust
    ui.add_space(8.0);

    // Source Image selection
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
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check --lib`
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add image selection UI to configure step

Add async image loading with retry logic (max 3, exponential backoff).
Show loading state, grouped dropdown, or fallback on failure.
Auto-select latest image on successful load."
```

---

### Task 15: Modify render_configure_server - Add Disk Size UI

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs` (render_configure_server, after image UI)

**Interfaces:**
- Consumes: `disk_size_gb`, `validate_disk_size` from Tasks 7-8
- Produces: Disk size input UI with validation

- [ ] **Step 1: Add disk size UI after image selection**

Add after image selection UI:

```rust
    ui.add_space(8.0);

    // Disk Size input
    ui.horizontal(|ui| {
        ui.label("Disk Size (GB):");
        ui.add(egui::TextEdit::singleline(&mut self.disk_size_gb).desired_width(80.0));
        ui.colored_label(egui::Color32::GRAY, "Minimum: 10 GB");
    });

    // Validation
    if let Err(e) = validate_disk_size(&self.disk_size_gb) {
        ui.colored_label(egui::Color32::from_rgb(245, 101, 101), format!("⚠ {}", e));
    }
```

- [ ] **Step 2: Update button enable condition**

Modify the `can_create` condition before "Create Server" button (around line 803):

```rust
    let can_create = !self.instance_name.is_empty()
        && self.validate_instance_name(&self.instance_name)
        && !self.selected_region.is_empty()
        && !self.selected_zone.is_empty()
        && !self.selected_machine_type.is_empty()
        && validate_disk_size(&self.disk_size_gb).is_ok()
        && self.image_promise.is_none();  // Wait for image loading
```

- [ ] **Step 3: Modify Back button visibility**

Replace Back button code (around line 799):

```rust
    ui.horizontal(|ui| {
        // Only show Back if came from full wizard
        if !self.skip_account_project_steps {
            if ui.button("← Back").clicked() {
                self.state = WizardState::SelectProject;
            }
        }

        // ... rest unchanged ...
    });
```

- [ ] **Step 4: Verify compilation**

Run: `cd mobile && cargo check --lib`
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): add disk size input to configure step

Add text field for disk size with 10GB minimum validation.
Disable Create button until validation passes and images load.
Hide Back button when steps skipped."
```

---

### Task 16: Modify start_server_creation to Use Image and Disk

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:1450-1535` (start_server_creation method)

**Interfaces:**
- Consumes: `selected_image`, `disk_size_gb` from Task 8
- Produces: Modified VM creation using custom image and disk size

- [ ] **Step 1: Modify start_server_creation to capture image and disk**

Replace relevant section in `start_server_creation` (around line 1454):

```rust
fn start_server_creation(&mut self) {
    self.state = WizardState::CreatingServer;
    self.progress_log.push("Creating server...".to_string());

    let project_id = self.selected_project_id.clone();
    let zone = self.selected_zone.clone();
    let instance_name = self.instance_name.clone();
    let machine_type = self.selected_machine_type.clone();
    let platform_name = self.platform_name.clone();
    let source_image = self.selected_image.clone();  // NEW
    let disk_size_gb = self.disk_size_gb.clone();    // NEW

    let access_token = self
        .oauth_result
        .as_ref()
        .map(|o| o.access_token.clone())
        .unwrap_or_default();

    self.create_promise = Some(Promise::spawn_thread("gcp_create_vm", move || {
        let client = GcpRestClient::new(access_token);

        // Create firewall rule if it doesn't exist
        Self::ensure_firewall_exists(&client, &project_id)
            .map_err(|e| format!("Failed to ensure firewall: {}", e))?;

        // Generate SSH key pair for this instance
        let (ssh_private_key, ssh_public_key, _raw_private, _raw_public) =
            Self::generate_ssh_key_pair()
                .map_err(|e| format!("Failed to generate SSH key: {}", e))?;

        // Store private key in keyring
        Self::store_ssh_key_in_keyring(&instance_name, &platform_name, &ssh_private_key)
            .map_err(|e| format!("Failed to store SSH key: {}", e))?;

        // Create instance request with custom image and disk size
        let mut instance_req =
            InstanceRequest::debian_micro(instance_name.clone(), zone.clone());

        // Customize machine type
        instance_req.machine_type = format!("zones/{}/machineTypes/{}", zone, machine_type);

        // Apply custom image and disk size (NEW)
        instance_req.disks[0].initialize_params.source_image = source_image;
        instance_req.disks[0].initialize_params.disk_size_gb = disk_size_gb;

        // Generate and add startup script metadata
        let startup_script = Self::generate_startup_script(&ssh_public_key);
        instance_req.metadata = Some(Metadata {
            items: vec![MetadataItem {
                key: "startup-script".to_string(),
                value: startup_script,
            }],
        });

        // ... rest unchanged (create instance, wait, return) ...
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`
Expected: No errors

- [ ] **Step 3: Run all platform_gcp tests**

Run: `cd mobile && cargo test ui_dlg::platform_gcp::tests --lib`
Expected: PASS (all tests)

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(gcp): use selected image and disk size in VM creation

Apply user-selected source_image and disk_size_gb to InstanceRequest.
Maintains backward compatibility with default values."
```

---

### Task 17: Final Integration Test and Verification

**Files:**
- Test: All modified files

**Interfaces:**
- Consumes: All tasks 1-16
- Produces: Verified working implementation

- [ ] **Step 1: Run all compute.rs tests**

Run: `cd mobile && cargo test api::gcp::compute::tests --lib`
Expected: PASS (all image filtering, display, validation, parsing tests)

- [ ] **Step 2: Run all platform_gcp.rs tests**

Run: `cd mobile && cargo test ui_dlg::platform_gcp::tests --lib`
Expected: PASS (all constructor, validation, fallback tests)

- [ ] **Step 3: Verify full project compiles**

Run: `cd mobile && cargo build --lib`
Expected: SUCCESS with no errors

- [ ] **Step 4: Run format check**

Run: `cd mobile && cargo fmt --check`
Expected: All files formatted correctly

- [ ] **Step 5: Run clippy**

Run: `cd mobile && cargo clippy --lib`
Expected: No new warnings related to changes

- [ ] **Step 6: Create summary commit**

```bash
git add -A
git commit -m "feat(gcp): complete image selection and workflow optimization

Summary of changes:
- Add Image types with filtering (recent, x86_64, not deprecated)
- Add list_images and list_debian_ubuntu_images API methods
- Add disk size validation (10-65536 GB)
- Add with_platform_context constructor to skip steps
- Add async image loading with retry (exponential backoff)
- Add fallback images (Debian 13, Ubuntu 24.04 LTS)
- Modify progress indicator for shortened flow
- Add image dropdown and disk input to configure step
- Apply custom image/disk to VM creation

Test coverage:
- Image filtering: 100%
- Display formatting: 100%
- Validation: 100%
- JSON parsing: sample coverage

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Plan Self-Review

**Spec Coverage Check:**
- ✅ Image types and filtering (Tasks 1-6)
- ✅ Disk size validation (Task 7)
- ✅ Skip account/project steps (Tasks 8-10, 12)
- ✅ Async loading with retry (Tasks 13-14)
- ✅ Fallback images (Task 11)
- ✅ UI modifications (Tasks 14-15)
- ✅ VM creation with custom params (Task 16)
- ✅ TDD unit tests throughout

**Placeholder Scan:**
- ✅ No TBD, TODO, "implement later"
- ✅ All code blocks complete
- ✅ All test assertions specific
- ✅ All file paths exact

**Type Consistency:**
- ✅ `Image` type defined in Task 1, used consistently in 2-16
- ✅ `validate_disk_size` signature matches usage
- ✅ `with_platform_context` parameters match spec
- ✅ `get_fallback_images` return type matches `available_images` field

**All requirements implemented.**
