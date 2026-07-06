# Docker Dialogs Redesign - Design Spec

**Date:** 2026-07-06  
**Status:** Approved  
**Goal:** Reimplement Docker image installation as a two-step wizard with automatic port/env detection, and add container removal dialog

## Problem Statement

The current Docker image installation dialog has three limitations:

1. **Manual configuration burden** - Users must manually configure port mappings and environment variables without knowing what the image actually exposes
2. **Docker Hub API dependency** - Image validation uses Docker Hub API rather than inspecting the actual image on the remote host
3. **No container removal UI** - Users can uninstall Docker daemon but cannot remove individual containers

## Solution Overview

**Install Docker Image Dialog - Two-Step Wizard:**
1. **Step 1:** User enters image ID → run `docker pull` + `docker history` to extract ExposedPorts and Env
2. **Step 2:** Pre-fill port mappings and environment variables based on inspection, allow editing, then install

**Remove Containers Dialog:**
- Replace "Uninstall Docker" button with "Remove Containers" button
- Show list of containers with checkboxes for multi-select
- Delete selected containers with error reporting per container

## Architecture

### Component Changes

**UI Layer (`mobile/src/ui_tabs/ssh.rs`):**
- Replace `render_docker_install_dialog()` with two-step wizard:
  - Step 1: Image input + inspection (spinner during pull/history)
  - Step 2: Port/env configuration (pre-filled, editable)
- Add new `render_docker_remove_dialog()`:
  - Shows list of containers with checkboxes
  - Single "Delete Selected" button
  - Results display with success/failure breakdown
- Replace "Uninstall Docker" button with "Remove Containers" button in operations row

**ViewModel Layer (`mobile/src/viewmodel/mod.rs` + `ssh/` module):**
- Add `InspectDockerImage` command (new)
- Enhance `RemoveDockerContainer` to `RemoveDockerContainers` for batch operations
- Keep `InstallDockerImage` unchanged (already supports ports/env)
- Remove `ValidateDockerImage` command (no longer needed)

**Actor Layer (`mobile/src/viewmodel/ssh/actor.rs`):**
- Implement `InspectDockerImage` handler:
  - SSH to host
  - Run `docker pull <image>`
  - Run `docker history <image> --no-trunc --format "{{.CreatedBy}}"`
  - Parse output to extract EXPOSE and ENV directives
  - Return structured data via `DockerImageInspected` event
- Implement `RemoveDockerContainers` handler:
  - SSH to host
  - Run `docker rm <container>` for each selected container
  - Return success/failure per container via `DockerContainersRemoved` event

### New Commands

```rust
// In mobile/src/viewmodel/ssh/commands.rs

pub enum SshCommand {
    // ... existing commands ...
    
    /// Inspect Docker image by pulling and analyzing history
    InspectDockerImage {
        host_name: String,
        image: String,
        tag: String,
    },
    
    /// Remove multiple Docker containers (batch operation)
    RemoveDockerContainers {
        host_name: String,
        container_names: Vec<String>,
    },
}
```

### New Events

```rust
// In mobile/src/viewmodel/ssh/events.rs

pub enum SshEvent {
    // ... existing events ...
    
    /// Docker image inspection completed
    DockerImageInspected {
        image: String,
        tag: String,
        exposed_ports: Vec<u16>,
        env_vars: Vec<(String, String)>,
    },
    
    /// Docker containers removed (batch operation)
    DockerContainersRemoved {
        host_name: String,
        removed: Vec<String>,           // successfully removed
        failed: Vec<(String, String)>,  // (container_name, error_message)
    },
}
```

## Install Docker Image Dialog - Two-Step Wizard

### State Variables

Add to `SshTab` struct:

```rust
// Dialog visibility
show_docker_install_dialog: bool,
docker_install_host_idx: Option<usize>,

// Step tracking
docker_install_step: u8,  // 1 or 2

// Step 1: Image input
docker_image_input: String,
docker_inspecting: bool,
docker_inspect_error: Option<String>,

// Step 2: Configuration (from inspection)
docker_container_name: String,         // auto-generated
docker_parsed_image: String,           // e.g., "linuxserver/wireguard"
docker_parsed_tag: String,             // e.g., "latest" or "v1.2.3"
docker_exposed_ports: Vec<u16>,        // from EXPOSE directives
docker_env_vars: Vec<(String, String)>, // from ENV directives

// Step 2: User-editable mappings
docker_port_mappings: Vec<(String, String)>,  // (host, container)
docker_env_overrides: Vec<(String, String)>,  // editable copy

// Installation progress
docker_installing: bool,
docker_install_success: bool,
docker_install_error: Option<String>,
```

### Step 1: Image Inspection

**UI Layout:**
```
┌─────────────────────────────────────────┐
│ Install Docker Image                  × │
├─────────────────────────────────────────┤
│                                         │
│ Image: [linuxserver/wireguard:latest ] │
│ Format: owner/image or owner/image:tag  │
│                                         │
│ [Inspect Image]  [Cancel]              │
│                                         │
│ (spinner appears during inspection)     │
│ ✓ Image pulled successfully             │
│ ✓ Found 2 exposed ports, 5 env vars    │
│                                         │
└─────────────────────────────────────────┘
```

**Flow:**
1. User enters image (e.g., `linuxserver/wireguard:latest`)
2. Parse into `image:tag`:
   - If no `:` present → default tag to `latest`
   - Split on last `:` → `image` and `tag`
3. User clicks "Inspect Image" button
4. Set `docker_inspecting = true`, show spinner
5. Send `InspectDockerImage { host_name, image, tag }` command
6. Actor performs:
   ```bash
   docker pull <image>:<tag>
   docker history <image>:<tag> --no-trunc --format "{{.CreatedBy}}"
   ```
7. On `DockerImageInspected` event:
   - Auto-generate container name: `<base_image>-1` (e.g., `wireguard-1`)
   - Pre-fill `docker_port_mappings` from `exposed_ports` (host=container for each)
   - Pre-fill `docker_env_overrides` from `env_vars`
   - Set `docker_install_step = 2`
8. On error: Show error message, stay in step 1, allow retry

### Step 2: Configuration & Installation

**UI Layout:**
```
┌──────────────────────────────────────────────┐
│ Install Docker Image                       × │
├──────────────────────────────────────────────┤
│                                              │
│ Image: linuxserver/wireguard:latest         │
│                                              │
│ Container Name: [wireguard-1            ]   │
│                                              │
│ Port Mappings:                               │
│ ┌────────────────────────────────────────┐  │
│ │ Host: [51820] → Container: [51820]  [−]│  │
│ └────────────────────────────────────────┘  │
│ [+ Add Port Mapping]                         │
│                                              │
│ Environment Variables:                       │
│ ┌────────────────────────────────────────┐  │
│ │ [PUID       ] = [1000          ]     [−]│  │
│ │ [PGID       ] = [1000          ]     [−]│  │
│ │ [TZ         ] = [Etc/UTC       ]     [−]│  │
│ └────────────────────────────────────────┘  │
│ [+ Add Environment Variable]                 │
│                                              │
│ [Back]  [Install]  [Cancel]                 │
│                                              │
│ (after install succeeds)                     │
│ ✓ Container installed successfully          │
│ [Close]                                      │
└──────────────────────────────────────────────┘
```

**Flow:**
1. Display:
   - Image:tag as read-only label
   - Container name in editable text field
   - Port mappings table (editable, add/remove rows)
   - Environment variables table (editable, add/remove rows)
2. User edits configuration as needed
3. User clicks "Install" button
4. Validate:
   - Container name not empty
   - Port numbers are valid (1-65535)
5. Send `InstallDockerImage` command with:
   ```rust
   InstallDockerImage {
       host_name,
       container_name,
       image: parsed_image,
       tag: parsed_tag,
       ports: docker_port_mappings.parse(),
       env: docker_env_overrides.filter(non-empty),
   }
   ```
6. Set `docker_installing = true`, show spinner
7. On success:
   - Set `docker_install_success = true`
   - Show "✓ Container installed successfully"
   - Show "Close" button only
8. On error:
   - Set `docker_install_error = Some(message)`
   - Show error, keep "Install" button enabled for retry

**Navigation:**
- Step 2 → Step 1: "Back" button resets state, returns to image input
- Close button: Always available, closes dialog and resets all state
- Success state: Only "Close" button visible

## Remove Containers Dialog

### State Variables

Add to `SshTab` struct:

```rust
// Dialog visibility
show_docker_remove_dialog: bool,
docker_remove_host_idx: Option<usize>,

// Container list
docker_available_containers: Vec<DockerContainerConfig>,
docker_selected_containers: Vec<String>,  // container names

// Operation state
docker_fetching_containers: bool,
docker_fetch_error: Option<String>,
docker_removing: bool,
docker_remove_results: Option<RemoveResults>,
```

**RemoveResults Structure:**
```rust
#[derive(Clone, Debug)]
struct RemoveResults {
    removed: Vec<String>,              // successfully removed
    failed: Vec<(String, String)>,     // (container_name, error_message)
}
```

### Dialog Flow

**UI Layout - Initial Load:**
```
┌─────────────────────────────────────────┐
│ Remove Docker Containers              × │
├─────────────────────────────────────────┤
│                                         │
│ ⏳ Loading containers...                │
│                                         │
└─────────────────────────────────────────┘
```

**UI Layout - Container Selection:**
```
┌──────────────────────────────────────────────────┐
│ Remove Docker Containers                       × │
├──────────────────────────────────────────────────┤
│                                                  │
│ Select containers to remove:                     │
│                                                  │
│ [☑] wireguard-1                                  │
│     Image: linuxserver/wireguard:latest          │
│     Status: running                              │
│     Ports: 51820→51820/udp                       │
│                                                  │
│ [☐] nginx-1                                      │
│     Image: nginx:alpine                          │
│     Status: stopped                              │
│     Ports: 80→80, 443→443                        │
│                                                  │
│ [☐] redis-1                                      │
│     Image: redis:7                               │
│     Status: running                              │
│     Ports: 6379→6379                             │
│                                                  │
│ 1 of 3 containers selected                       │
│                                                  │
│ [Select All] [Deselect All]                      │
│                                                  │
│ [Delete Selected]  [Cancel]                      │
│                                                  │
└──────────────────────────────────────────────────┘
```

**UI Layout - Confirmation:**
```
│ Remove 2 containers? This cannot be undone.      │
│                                                  │
│ [Confirm]  [Cancel]                              │
```

**UI Layout - Results:**
```
│ ✓ Removed: wireguard-1                           │
│ ⚠ Failed: nginx-1 (Error: container is running) │
│                                                  │
│ [Close]                                          │
```

### Flow Steps

1. **Initial Load:**
   - Dialog opens
   - Set `docker_fetching_containers = true`, show spinner
   - Send `ListDockerContainers` command
   - On `DockerContainersListed` event:
     - Populate `docker_available_containers`
     - Initialize `docker_selected_containers` as empty
   - On error: Show error message with "Retry" button

2. **Container Selection:**
   - Each row renders:
     - Checkbox (bound to `docker_selected_containers`)
     - Container name (bold)
     - Image:tag
     - Status badge (color-coded: green=running, gray=stopped, red=error)
     - Port mappings (abbreviated)
   - "Select All" / "Deselect All" buttons
   - Selection count: "N of M containers selected"
   - "Delete Selected" button:
     - Disabled if `docker_selected_containers.is_empty()`
     - Red background when enabled

3. **Confirmation:**
   - On "Delete Selected" click:
     - Show inline confirmation message
     - "Remove N containers? This cannot be undone."
     - "Confirm" / "Cancel" buttons
   - On Cancel: Hide confirmation, return to selection
   - On Confirm: Proceed to deletion

4. **Deletion:**
   - Set `docker_removing = true`, show spinner
   - Show progress: "Removing 1 of 2..."
   - Send `RemoveDockerContainers` command with selected names
   - Actor performs:
     ```bash
     for container in containers:
         docker rm <container>  # NOT docker rm -f
     ```
   - Collect results (success vs failure per container)
   - On completion: Show results display

5. **Results Display:**
   - If all succeeded:
     - "✓ All N containers removed successfully"
   - If partial failure:
     - "✓ Removed: container1, container2"
     - "⚠ Failed: container3 (error: container is running)"
   - "Close" button to dismiss dialog
   - On close: Refresh container list in drawer (config updated)

### Button Replacement in Operations Row

**Current:**
```rust
// In render_operations() around line 2190
if row.docker_enabled {
    if ui.add(MaterialButton::outlined("Uninstall Docker").small()).clicked() {
        // ...
    }
}
```

**New:**
```rust
if row.docker_enabled && !row.docker_containers.is_empty() {
    if ui.add(MaterialButton::outlined("Remove Containers").small())
        .on_hover_text("Remove Docker containers")
        .clicked()
    {
        ui.data_mut(|d| {
            d.insert_temp(
                egui::Id::new(format!("ssh_remove_containers_{}", idx)),
                row.host.clone(),
            )
        });
    }
}
```

## Docker History Parsing

### Command Format

```bash
docker history <image>:<tag> --no-trunc --format "{{.CreatedBy}}"
```

### Sample Output

```
/bin/sh -c #(nop)  CMD ["/init"]
/bin/sh -c #(nop)  EXPOSE 51820/udp
/bin/sh -c #(nop)  ENV PUID=1000
/bin/sh -c #(nop)  ENV PGID=1000
/bin/sh -c #(nop)  ENV TZ=Etc/UTC
/bin/sh -c apt-get update && apt-get install -y wireguard
...
```

### Parsing Logic

Implement in `mobile/src/viewmodel/ssh/actor.rs`:

```rust
fn parse_docker_history(output: &str) -> (Vec<u16>, Vec<(String, String)>) {
    let mut ports = Vec::new();
    let mut env_vars = Vec::new();
    
    for line in output.lines() {
        let line = line.trim();
        
        // Parse EXPOSE directives
        // Format: "EXPOSE 8080/tcp" or "EXPOSE 51820/udp" or "EXPOSE 80"
        if let Some(expose_part) = line.strip_prefix("/bin/sh -c #(nop)  EXPOSE ") {
            if let Some(port_str) = expose_part.split('/').next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    ports.push(port);
                }
            }
        }
        
        // Parse ENV directives
        // Format: "ENV KEY=value" or "ENV KEY value"
        if let Some(env_part) = line.strip_prefix("/bin/sh -c #(nop)  ENV ") {
            if let Some((key, value)) = env_part.split_once('=') {
                env_vars.push((key.trim().to_string(), value.trim().to_string()));
            }
        }
    }
    
    // Remove duplicate ports
    ports.sort_unstable();
    ports.dedup();
    
    // For env vars, keep last occurrence (layers stack, last wins)
    let mut env_map: std::collections::HashMap<String, String> = 
        std::collections::HashMap::new();
    for (k, v) in env_vars.iter().rev() {
        env_map.entry(k.clone()).or_insert_with(|| v.clone());
    }
    env_vars = env_map.into_iter().collect();
    env_vars.sort_by(|a, b| a.0.cmp(&b.0));
    
    (ports, env_vars)
}
```

### Container Name Generation

Implement in `mobile/src/ui_tabs/ssh.rs`:

```rust
fn generate_container_name(
    image: &str, 
    existing_containers: &[crate::config::DockerContainerConfig]
) -> String {
    // Extract base name: "linuxserver/wireguard" → "wireguard"
    let base_name = image.split('/').last().unwrap_or(image);
    
    // Remove tag if present: "wireguard:latest" → "wireguard"
    let base_name = base_name.split(':').next().unwrap_or(base_name);
    
    // Find next available number
    let mut counter = 1;
    let mut name = format!("{}-{}", base_name, counter);
    while existing_containers.iter().any(|c| c.name == name) {
        counter += 1;
        name = format!("{}-{}", base_name, counter);
    }
    
    name
}
```

### Fallback Behavior

- **Empty ports/env from parsing:** Pre-fill with empty lists, user can add manually
- **`--format` flag not supported:** Try parsing table format (less reliable), fall back to empty
- **`docker history` command fails:** Show error, don't proceed to Step 2
- **Image name parse failure:** Show validation error in Step 1

## Error Handling

### Inspection Errors (Step 1)

| Error Case | User Experience |
|------------|-----------------|
| SSH connection fails | "⚠ Cannot connect to host. Check SSH configuration." |
| Image not found | "⚠ Image 'xyz' not found. Check the name and try again." |
| Docker not installed | "⚠ Docker is not installed on this host. Install Docker first." |
| `docker pull` fails (network) | "⚠ Failed to pull image: \<docker error message\>" |
| `docker history` parse failure | "⚠ Could not inspect image layers. The image may be corrupted." |
| No ports/env found | Pre-fill with empty lists (valid - user can add manually) |
| Image tag parse error | "⚠ Invalid image format. Use owner/image or owner/image:tag" |

### Installation Errors (Step 2)

| Error Case | User Experience |
|------------|-----------------|
| Invalid port mapping | "⚠ Port '8o8o' is not a valid number." (validate on input) |
| Port already in use | "⚠ Installation failed: Port 8080 is already in use." |
| Container name conflict | "⚠ Container 'wireguard-1' already exists. Choose a different name." |
| Container name invalid | "⚠ Container name must be alphanumeric with hyphens/underscores only." |
| Generic docker run error | "⚠ Failed to start container: \<docker error\>" |
| SSH connection lost | "⚠ Lost connection to host during installation." |

### Removal Errors

| Error Case | User Experience |
|------------|-----------------|
| SSH connection fails | "⚠ Cannot connect to host. Removal cancelled." |
| Container is running | Show in results: "⚠ Failed: container1 (Error: container is running)" |
| Container doesn't exist | Treat as success (already removed) |
| Partial batch failure | Show both successes and failures in results panel |
| Docker daemon error | "⚠ Docker error: \<message\>" |
| All containers failed | "⚠ Failed to remove any containers. Check error details." |

### Validation Rules

**Port Numbers:**
- Only allow digits (0-9)
- Range: 1-65535
- Validate on input (reject non-numeric characters)
- Empty host port allowed (will auto-assign)

**Container Names:**
- Pattern: `^[a-zA-Z0-9][a-zA-Z0-9_-]*$`
- No spaces, no special chars except `_` and `-`
- Must start with alphanumeric
- Max length: 255 characters

**Environment Variable Keys:**
- Pattern: `^[A-Z_][A-Z0-9_]*$`
- Convention: uppercase with underscores
- No spaces, no special chars
- Empty values allowed (user may want to unset defaults)

### Recovery Actions

- All errors keep the dialog open with options to:
  - Edit configuration and retry (install dialog)
  - Close and try again later
  - View full error details (expandable section with raw output)
- Progress events shown via ViewModel's active operations progress bars
- Errors logged to console with `eprintln!` for debugging

## Data Flow

### Install Dialog - Inspection Phase

```
UI (Step 1)                    ViewModel                   SSH Actor
    |                              |                            |
    | User enters image            |                            |
    | Clicks "Inspect"             |                            |
    |----------------------------->|                            |
    | InspectDockerImage command   |                            |
    |                              |-------------------------->|
    |                              | SshCommand::InspectImage   |
    |                              |                            |
    |                              |                     [SSH to host]
    |                              |                [docker pull <image>]
    |                              |             [docker history <image>]
    |                              |                    [Parse output]
    |                              |                            |
    |                              |<--------------------------|
    |                              | DockerImageInspected event |
    |<-----------------------------|                            |
    | Update state, show Step 2    |                            |
```

### Install Dialog - Installation Phase

```
UI (Step 2)                    ViewModel                   SSH Actor
    |                              |                            |
    | User clicks "Install"        |                            |
    |----------------------------->|                            |
    | InstallDockerImage command   |                            |
    |                              |-------------------------->|
    |                              | SshCommand::InstallImage   |
    |                              |                            |
    |                              |                     [SSH to host]
    |                              |             [docker run with config]
    |                              |                            |
    |                              |<--------------------------|
    |                              | DockerImageInstalled event |
    |<-----------------------------|                            |
    | Show success, update config  |                            |
```

### Remove Dialog

```
UI                             ViewModel                   SSH Actor
    |                              |                            |
    | Dialog opens                 |                            |
    |----------------------------->|                            |
    | ListDockerContainers command |                            |
    |                              |-------------------------->|
    |                              | SshCommand::ListContainers |
    |                              |                            |
    |                              |                     [SSH to host]
    |                              |              [docker ps -a --format]
    |                              |                            |
    |                              |<--------------------------|
    |                              | DockerContainersListed     |
    |<-----------------------------|                            |
    | Show checkboxes              |                            |
    |                              |                            |
    | User selects, clicks Delete  |                            |
    |----------------------------->|                            |
    | RemoveDockerContainers cmd   |                            |
    |                              |-------------------------->|
    |                              | SshCommand::RemoveContainers|
    |                              |                            |
    |                              |                [docker rm for each]
    |                              |                            |
    |                              |<--------------------------|
    |                              | DockerContainersRemoved    |
    |<-----------------------------|                            |
    | Show results (success/fails) |                            |
```

## State Persistence

**After Successful Install:**
1. Actor sends `DockerImageInstalled` event
2. UI handler updates `SshHostConfig.docker_containers` in config
3. Save config file to disk
4. Update `SshRowData.docker_containers` for drawer display
5. Dialog shows success, user closes

**After Successful Removal:**
1. Actor sends `DockerContainersRemoved` event with results
2. UI handler removes from `SshHostConfig.docker_containers` (only removed ones)
3. Save config file to disk
4. Update `SshRowData.docker_containers` for drawer display
5. Dialog shows results, user closes

**Source of Truth:**
- Config file is authoritative
- No need to call `ListDockerContainers` after install/remove
- Drawer refreshes automatically when row data updates

## Progress Tracking

**Inspection Phase:**
- Indeterminate spinner (duration: 1-60 seconds depending on image size)
- Text: "Pulling and inspecting image..."
- No percentage (pull doesn't report progress reliably)

**Installation Phase:**
- Reuse existing ViewModel progress event system
- Actor sends `Progress` events during `docker run`
- Show progress bar with status messages

**Removal Phase:**
- Batch progress indicator
- Text: "Removing 2 of 5 containers..."
- Update after each container processed

## Testing Checklist

### Install Dialog

- [ ] Image with explicit tag (e.g., `redis:7`) parses correctly
- [ ] Image without tag (e.g., `nginx`) defaults to `latest`
- [ ] Image not found shows appropriate error
- [ ] Docker not installed on host shows error
- [ ] Exposed ports pre-fill correctly (single port, multiple ports, TCP/UDP)
- [ ] Environment variables pre-fill correctly (single, multiple, with spaces in values)
- [ ] Container name auto-generates uniquely (wireguard-1, wireguard-2, etc.)
- [ ] Port mapping validation rejects non-numeric input
- [ ] Empty ports/env list allows manual configuration
- [ ] Back button resets state and returns to Step 1
- [ ] Success message shows with Close button only
- [ ] Install failure shows error and keeps Install button enabled
- [ ] Config file updates after successful install
- [ ] Drawer refreshes to show new container

### Remove Dialog

- [ ] Container list loads and displays correctly
- [ ] Checkboxes select/deselect containers
- [ ] "Select All" / "Deselect All" work correctly
- [ ] Delete button disabled when no containers selected
- [ ] Confirmation appears before deletion
- [ ] Cancel confirmation returns to selection
- [ ] Batch removal processes all selected containers
- [ ] Results show both successes and failures
- [ ] Running container removal fails with appropriate message
- [ ] Non-existent container treated as success
- [ ] Config file updates after successful removal
- [ ] Drawer refreshes to remove deleted containers
- [ ] "Remove Containers" button only shown when containers exist

### Edge Cases

- [ ] SSH connection lost during inspection
- [ ] SSH connection lost during installation
- [ ] SSH connection lost during removal
- [ ] Docker daemon not running
- [ ] Image pull timeout (very large image)
- [ ] Container name with special characters rejected
- [ ] Port already in use on host
- [ ] Container with same name already exists
- [ ] Rapid dialog open/close doesn't cause state corruption

## Implementation Notes

### Files to Modify

1. **`mobile/src/ui_tabs/ssh.rs`** (major changes)
   - Replace `render_docker_install_dialog()` with two-step version
   - Add `render_docker_remove_dialog()`
   - Update state variables
   - Add helper functions: `generate_container_name()`, validation
   - Replace "Uninstall Docker" button in `render_operations()`

2. **`mobile/src/viewmodel/ssh/commands.rs`** (new commands)
   - Add `InspectDockerImage` variant
   - Modify `RemoveDockerContainer` → `RemoveDockerContainers` (plural)

3. **`mobile/src/viewmodel/ssh/events.rs`** (new events)
   - Add `DockerImageInspected` variant
   - Add `DockerContainersRemoved` variant

4. **`mobile/src/viewmodel/ssh/actor.rs`** (command handlers)
   - Implement `handle_inspect_docker_image()`
   - Implement `handle_remove_docker_containers()`
   - Add helper: `parse_docker_history()`

5. **`mobile/src/viewmodel/mod.rs`** (ViewModel API)
   - Add `inspect_docker_image()` method
   - Modify `remove_docker_container()` to `remove_docker_containers()`
   - Remove `validate_docker_image()` method

### Dependencies

No new external dependencies required. Uses existing:
- `russh` for SSH commands
- `egui` + `egui_material3` for UI
- `smol::channel` for actor communication

## Success Criteria

1. ✅ Install dialog runs `docker pull` + `docker history` on remote host (not Docker Hub API)
2. ✅ Exposed ports and env vars pre-fill automatically in Step 2
3. ✅ Users can edit pre-filled configuration before installing
4. ✅ Container names auto-generate uniquely (image-1, image-2, etc.)
5. ✅ Remove dialog shows container list with multi-select checkboxes
6. ✅ Batch removal processes multiple containers with per-container error reporting
7. ✅ "Uninstall Docker" button replaced with "Remove Containers" button
8. ✅ All operations show appropriate progress and error feedback
9. ✅ Config file persists changes after install/remove operations
10. ✅ Drawer updates automatically to reflect current container state
