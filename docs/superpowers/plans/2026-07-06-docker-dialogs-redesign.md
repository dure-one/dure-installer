# Docker Dialogs Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reimplement Docker image installation as a two-step wizard with automatic port/env detection via `docker pull` + `docker history`, and add container removal dialog with multi-select

**Architecture:** Two-phase approach - first add new commands/events/handlers at the actor level, then rebuild UI dialogs to use them. Install dialog becomes a two-step wizard (inspect → configure → install). Remove dialog shows container list with checkboxes for batch deletion.

**Tech Stack:** Rust, egui, egui-material3, russh for SSH, smol for async

## Global Constraints

- Rust nightly toolchain required
- Follow existing MVVM pattern: UI → ViewModel → Actor
- All SSH commands via russh (no OpenSSL)
- egui immediate mode - dialog state rebuilt each frame
- Config file (`config.yaml`) is source of truth for containers
- Use TDD: write failing test, implement minimal code, verify pass
- Commit after each task completes

---

### Task 1: Add New Commands and Events

**Files:**
- Modify: `mobile/src/viewmodel/ssh/commands.rs:3-102`
- Modify: `mobile/src/viewmodel/ssh/events.rs:23-209`

**Interfaces:**
- Consumes: Existing `SshCommand` and `SshEvent` enums
- Produces: `InspectDockerImage`, `RemoveDockerContainers` commands; `DockerImageInspected`, `DockerContainersRemoved` events

- [ ] **Step 1: Add InspectDockerImage command variant**

In `mobile/src/viewmodel/ssh/commands.rs`, add after `ListDockerContainers` (around line 61):

```rust
/// Inspect Docker image by pulling and analyzing history
InspectDockerImage {
    host_name: String,
    image: String,
    tag: String,
},
```

- [ ] **Step 2: Add RemoveDockerContainers command variant**

Add after `RemoveDockerContainer` (around line 58):

```rust
/// Remove multiple Docker containers (batch operation)
RemoveDockerContainers {
    host_name: String,
    container_names: Vec<String>,
},
```

- [ ] **Step 3: Add DockerImageInspected event variant**

In `mobile/src/viewmodel/ssh/events.rs`, add after `DureWssStatusRetrieved` (around line 189):

```rust
/// Docker image inspection completed
DockerImageInspected {
    image: String,
    tag: String,
    exposed_ports: Vec<u16>,
    env_vars: Vec<(String, String)>,
},
```

- [ ] **Step 4: Add DockerContainersRemoved event variant**

Add after `DockerImageInspected`:

```rust
/// Docker containers removed (batch operation)
DockerContainersRemoved {
    host_name: String,
    removed: Vec<String>,           // successfully removed
    failed: Vec<(String, String)>,  // (container_name, error_message)
},
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds with no errors

- [ ] **Step 6: Commit changes**

```bash
git add mobile/src/viewmodel/ssh/commands.rs mobile/src/viewmodel/ssh/events.rs
git commit -m "feat(ssh): add InspectDockerImage and RemoveDockerContainers commands/events

Add new command variants for image inspection and batch container removal.
Add corresponding event variants for inspection results and removal results.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Implement Docker History Parsing

**Files:**
- Modify: `mobile/src/viewmodel/ssh/actor.rs:1-end`

**Interfaces:**
- Consumes: Nothing (standalone utility function)
- Produces: `parse_docker_history(output: &str) -> (Vec<u16>, Vec<(String, String)>)`

- [ ] **Step 1: Add parsing function**

In `mobile/src/viewmodel/ssh/actor.rs`, add at the end before the impl block:

```rust
/// Parse docker history output to extract EXPOSE and ENV directives
fn parse_docker_history(output: &str) -> (Vec<u16>, Vec<(String, String)>) {
    let mut ports = Vec::new();
    let mut env_vars = Vec::new();
    
    for line in output.lines() {
        let line = line.trim();
        
        // Parse EXPOSE directives
        // Format: "/bin/sh -c #(nop)  EXPOSE 8080/tcp" or "EXPOSE 51820/udp" or "EXPOSE 80"
        if let Some(expose_part) = line.strip_prefix("/bin/sh -c #(nop)  EXPOSE ") {
            if let Some(port_str) = expose_part.split('/').next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    ports.push(port);
                }
            }
        }
        
        // Parse ENV directives
        // Format: "/bin/sh -c #(nop)  ENV KEY=value"
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
    use std::collections::HashMap;
    let mut env_map: HashMap<String, String> = HashMap::new();
    for (k, v) in env_vars.iter().rev() {
        env_map.entry(k.clone()).or_insert_with(|| v.clone());
    }
    env_vars = env_map.into_iter().collect();
    env_vars.sort_by(|a, b| a.0.cmp(&b.0));
    
    (ports, env_vars)
}
```

- [ ] **Step 2: Add unit test for empty output**

Add after `parse_docker_history` function:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_history_empty() {
        let output = "";
        let (ports, env_vars) = parse_docker_history(output);
        assert!(ports.is_empty());
        assert!(env_vars.is_empty());
    }
}
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cargo test --lib -p dure parse_docker_history_empty`
Expected: PASS

- [ ] **Step 4: Add test for parsing ports**

Add to tests module:

```rust
#[test]
fn test_parse_docker_history_ports() {
    let output = r#"/bin/sh -c #(nop)  CMD ["/init"]
/bin/sh -c #(nop)  EXPOSE 51820/udp
/bin/sh -c #(nop)  EXPOSE 8080/tcp
/bin/sh -c #(nop)  EXPOSE 80"#;
    
    let (ports, env_vars) = parse_docker_history(output);
    assert_eq!(ports, vec![80, 8080, 51820]);
    assert!(env_vars.is_empty());
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib -p dure parse_docker_history_ports`
Expected: PASS

- [ ] **Step 6: Add test for parsing env vars**

Add to tests module:

```rust
#[test]
fn test_parse_docker_history_env_vars() {
    let output = r#"/bin/sh -c #(nop)  ENV PUID=1000
/bin/sh -c #(nop)  ENV PGID=1000
/bin/sh -c #(nop)  ENV TZ=Etc/UTC"#;
    
    let (ports, env_vars) = parse_docker_history(output);
    assert!(ports.is_empty());
    assert_eq!(env_vars.len(), 3);
    
    // Check that PGID exists (order may vary due to HashMap)
    assert!(env_vars.iter().any(|(k, v)| k == "PGID" && v == "1000"));
    assert!(env_vars.iter().any(|(k, v)| k == "PUID" && v == "1000"));
    assert!(env_vars.iter().any(|(k, v)| k == "TZ" && v == "Etc/UTC"));
}
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test --lib -p dure parse_docker_history_env_vars`
Expected: PASS

- [ ] **Step 8: Add test for deduplication**

Add to tests module:

```rust
#[test]
fn test_parse_docker_history_deduplication() {
    let output = r#"/bin/sh -c #(nop)  EXPOSE 8080/tcp
/bin/sh -c #(nop)  EXPOSE 8080/tcp
/bin/sh -c #(nop)  ENV PATH=/usr/bin
/bin/sh -c #(nop)  ENV PATH=/usr/local/bin"#;
    
    let (ports, env_vars) = parse_docker_history(output);
    assert_eq!(ports, vec![8080]); // deduplicated
    assert_eq!(env_vars.len(), 1);
    assert_eq!(env_vars[0].0, "PATH");
    // Last occurrence wins
    assert_eq!(env_vars[0].1, "/usr/local/bin");
}
```

- [ ] **Step 9: Run test to verify it passes**

Run: `cargo test --lib -p dure parse_docker_history_deduplication`
Expected: PASS

- [ ] **Step 10: Commit changes**

```bash
git add mobile/src/viewmodel/ssh/actor.rs
git commit -m "feat(ssh): add docker history parsing with tests

Implement parse_docker_history() to extract EXPOSE and ENV directives
from docker history output. Handles deduplication and layer stacking.

Tests cover: empty output, ports, env vars, deduplication.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Implement InspectDockerImage Actor Handler

**Files:**
- Modify: `mobile/src/viewmodel/ssh/actor.rs:1-end`

**Interfaces:**
- Consumes: `SshCommand::InspectDockerImage`, `parse_docker_history()` function
- Produces: `SshEvent::DockerImageInspected` event via event sender

- [ ] **Step 1: Find command handler match block**

Locate the `match cmd` block in `SshActor::run()` where commands are handled (search for `SshCommand::ListHosts =>` to find the pattern)

- [ ] **Step 2: Add InspectDockerImage handler skeleton**

Add after `ListDockerContainers` handler:

```rust
SshCommand::InspectDockerImage { host_name, image, tag } => {
    eprintln!("🔍 SSH Actor: inspect_docker_image called for '{}' with {}:{}", host_name, image, tag);
    // TODO: Implement
}
```

- [ ] **Step 3: Implement docker pull command**

Replace TODO with:

```rust
let full_image = format!("{}:{}", image, tag);

// Step 1: Pull the image
let pull_cmd = format!("docker pull {}", full_image);
match self.execute_ssh_command(&host_name, &pull_cmd).await {
    Ok(output) => {
        eprintln!("🔍 SSH Actor: Image pulled successfully");
        eprintln!("📋 Pull output: {}", output);
    }
    Err(e) => {
        eprintln!("❌ SSH Actor: Failed to pull image: {}", e);
        let _ = event_tx.send(ViewModelEvent::Ssh(SshEvent::Error {
            operation: format!("inspect_docker_image({})", full_image),
            error: format!("Failed to pull image: {}", e),
        })).await;
        continue;
    }
}
```

- [ ] **Step 4: Implement docker history command**

Add after pull command:

```rust
// Step 2: Get image history
let history_cmd = format!("docker history {} --no-trunc --format \"{{{{.CreatedBy}}}}\"", full_image);
let history_output = match self.execute_ssh_command(&host_name, &history_cmd).await {
    Ok(output) => output,
    Err(e) => {
        eprintln!("❌ SSH Actor: Failed to get image history: {}", e);
        let _ = event_tx.send(ViewModelEvent::Ssh(SshEvent::Error {
            operation: format!("inspect_docker_image({})", full_image),
            error: format!("Failed to inspect image history: {}", e),
        })).await;
        continue;
    }
};
```

- [ ] **Step 5: Parse history and send event**

Add after history command:

```rust
// Step 3: Parse history output
let (exposed_ports, env_vars) = parse_docker_history(&history_output);

eprintln!("🔍 SSH Actor: Sending DockerImageInspected event");
eprintln!("  Image: {}:{}", image, tag);
eprintln!("  Ports: {:?}", exposed_ports);
eprintln!("  Env vars: {} variables", env_vars.len());

let _ = event_tx.send(ViewModelEvent::Ssh(SshEvent::DockerImageInspected {
    image: image.clone(),
    tag: tag.clone(),
    exposed_ports,
    env_vars,
})).await;
eprintln!("✓ SSH Actor: DockerImageInspected event sent");
```

- [ ] **Step 6: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 7: Commit changes**

```bash
git add mobile/src/viewmodel/ssh/actor.rs
git commit -m "feat(ssh): implement InspectDockerImage actor handler

Execute docker pull and docker history on remote host via SSH.
Parse output using parse_docker_history() and send DockerImageInspected event.

Handles errors from pull and history commands with appropriate event messages.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Implement RemoveDockerContainers Actor Handler

**Files:**
- Modify: `mobile/src/viewmodel/ssh/actor.rs:1-end`

**Interfaces:**
- Consumes: `SshCommand::RemoveDockerContainers`
- Produces: `SshEvent::DockerContainersRemoved` event

- [ ] **Step 1: Add RemoveDockerContainers handler skeleton**

Add after `RemoveDockerContainer` handler in the match block:

```rust
SshCommand::RemoveDockerContainers { host_name, container_names } => {
    eprintln!("🔍 SSH Actor: remove_docker_containers called for '{}'", host_name);
    eprintln!("  Containers to remove: {:?}", container_names);
    // TODO: Implement
}
```

- [ ] **Step 2: Implement batch removal with result tracking**

Replace TODO with:

```rust
let mut removed = Vec::new();
let mut failed = Vec::new();

for container_name in container_names {
    let rm_cmd = format!("docker rm {}", container_name);
    match self.execute_ssh_command(&host_name, &rm_cmd).await {
        Ok(_) => {
            eprintln!("✓ SSH Actor: Removed container '{}'", container_name);
            removed.push(container_name.clone());
        }
        Err(e) => {
            eprintln!("❌ SSH Actor: Failed to remove '{}': {}", container_name, e);
            failed.push((container_name.clone(), e.to_string()));
        }
    }
}
```

- [ ] **Step 3: Send results event**

Add after removal loop:

```rust
eprintln!("🔍 SSH Actor: Sending DockerContainersRemoved event");
eprintln!("  Removed: {} containers", removed.len());
eprintln!("  Failed: {} containers", failed.len());

let _ = event_tx.send(ViewModelEvent::Ssh(SshEvent::DockerContainersRemoved {
    host_name: host_name.clone(),
    removed,
    failed,
})).await;
eprintln!("✓ SSH Actor: DockerContainersRemoved event sent");
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 5: Commit changes**

```bash
git add mobile/src/viewmodel/ssh/actor.rs
git commit -m "feat(ssh): implement RemoveDockerContainers actor handler

Execute docker rm for each container in the batch.
Track successes and failures separately.
Send DockerContainersRemoved event with results.

Allows partial success - some containers may be removed while others fail.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Add ViewModel API Methods

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs:1-end`

**Interfaces:**
- Consumes: `SshCommand` sender channel
- Produces: `inspect_docker_image()`, `remove_docker_containers()` methods; removes `validate_docker_image()`

- [ ] **Step 1: Find existing Docker methods**

Locate methods like `install_docker_image()` and `validate_docker_image()` in the ViewModel impl block

- [ ] **Step 2: Add inspect_docker_image method**

Add after `install_docker_image()`:

```rust
/// Inspect Docker image by pulling and analyzing history
pub fn inspect_docker_image(&self, host: String, image: String, tag: String) -> anyhow::Result<()> {
    self.ssh_tx.try_send(ssh::SshCommand::InspectDockerImage {
        host_name: host,
        image,
        tag,
    })?;
    Ok(())
}
```

- [ ] **Step 3: Add remove_docker_containers method**

Add after `remove_docker_container()`:

```rust
/// Remove multiple Docker containers in batch
pub fn remove_docker_containers(&self, host: String, container_names: Vec<String>) -> anyhow::Result<()> {
    self.ssh_tx.try_send(ssh::SshCommand::RemoveDockerContainers {
        host_name: host,
        container_names,
    })?;
    Ok(())
}
```

- [ ] **Step 4: Comment out or remove validate_docker_image method**

Find `pub fn validate_docker_image()` and comment it out:

```rust
// No longer used - inspection happens on remote host via docker pull/history
// pub fn validate_docker_image(&self, image: String) -> anyhow::Result<()> {
//     ...
// }
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds (may have warnings about unused validate_docker_image)

- [ ] **Step 6: Commit changes**

```bash
git add mobile/src/viewmodel/mod.rs
git commit -m "feat(ssh): add ViewModel API for image inspection and batch removal

Add inspect_docker_image() to trigger docker pull + history inspection.
Add remove_docker_containers() for batch container removal.
Deprecate validate_docker_image() (no longer needed).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Update SshTab State Variables

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:69-140`

**Interfaces:**
- Consumes: Existing `SshTab` struct
- Produces: New state variables for two-step wizard and removal dialog

- [ ] **Step 1: Locate Docker Install Dialog state variables**

Find the section with `show_docker_install_dialog`, `docker_image_input`, etc. (around line 97-117)

- [ ] **Step 2: Replace old state with new wizard state**

Replace the Docker Install Dialog section with:

```rust
// Docker Install Dialog (Two-Step Wizard)
#[cfg_attr(feature = "serde", serde(skip))]
show_docker_install_dialog: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_install_host_idx: Option<usize>,

// Step tracking
#[cfg_attr(feature = "serde", serde(skip))]
docker_install_step: u8,  // 1 or 2

// Step 1: Image input
#[cfg_attr(feature = "serde", serde(skip))]
docker_image_input: String,
#[cfg_attr(feature = "serde", serde(skip))]
docker_inspecting: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_inspect_error: Option<String>,

// Step 2: Configuration (from inspection)
#[cfg_attr(feature = "serde", serde(skip))]
docker_container_name: String,
#[cfg_attr(feature = "serde", serde(skip))]
docker_parsed_image: String,
#[cfg_attr(feature = "serde", serde(skip))]
docker_parsed_tag: String,
#[cfg_attr(feature = "serde", serde(skip))]
docker_exposed_ports: Vec<u16>,
#[cfg_attr(feature = "serde", serde(skip))]
docker_env_vars: Vec<(String, String)>,

// Step 2: User-editable mappings
#[cfg_attr(feature = "serde", serde(skip))]
docker_port_mappings: Vec<(String, String)>,  // (host, container)
#[cfg_attr(feature = "serde", serde(skip))]
docker_env_overrides: Vec<(String, String)>,  // editable copy

// Installation progress
#[cfg_attr(feature = "serde", serde(skip))]
docker_installing: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_install_success: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_install_error: Option<String>,

// Remove old validation-related fields:
// docker_tag: String,  // REMOVE
// docker_metadata: Option<...>,  // REMOVE
// docker_validating: bool,  // REMOVE
// docker_validation_error: Option<String>,  // REMOVE
```

- [ ] **Step 3: Add Remove Containers Dialog state variables**

Add after Docker Install Dialog section:

```rust
// Docker Remove Containers Dialog
#[cfg_attr(feature = "serde", serde(skip))]
show_docker_remove_dialog: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_remove_host_idx: Option<usize>,

// Container list
#[cfg_attr(feature = "serde", serde(skip))]
docker_available_containers: Vec<crate::config::DockerContainerConfig>,
#[cfg_attr(feature = "serde", serde(skip))]
docker_selected_containers: Vec<String>,  // container names

// Operation state
#[cfg_attr(feature = "serde", serde(skip))]
docker_fetching_containers: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_fetch_error: Option<String>,
#[cfg_attr(feature = "serde", serde(skip))]
docker_removing: bool,
#[cfg_attr(feature = "serde", serde(skip))]
docker_remove_results: Option<RemoveResults>,
```

- [ ] **Step 4: Add RemoveResults struct**

Add before `SshTab` struct definition:

```rust
/// Results from batch container removal
#[derive(Clone, Debug)]
struct RemoveResults {
    removed: Vec<String>,              // successfully removed
    failed: Vec<(String, String)>,     // (container_name, error_message)
}
```

- [ ] **Step 5: Update Default implementation**

Find the `impl Default for SshTab` and update it to initialize new fields:

```rust
impl Default for SshTab {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            
            // Docker Install Dialog (new)
            show_docker_install_dialog: false,
            docker_install_host_idx: None,
            docker_install_step: 1,
            docker_image_input: String::new(),
            docker_inspecting: false,
            docker_inspect_error: None,
            docker_container_name: String::new(),
            docker_parsed_image: String::new(),
            docker_parsed_tag: String::new(),
            docker_exposed_ports: Vec::new(),
            docker_env_vars: Vec::new(),
            docker_port_mappings: Vec::new(),
            docker_env_overrides: Vec::new(),
            docker_installing: false,
            docker_install_success: false,
            docker_install_error: None,
            
            // Docker Remove Containers Dialog (new)
            show_docker_remove_dialog: false,
            docker_remove_host_idx: None,
            docker_available_containers: Vec::new(),
            docker_selected_containers: Vec::new(),
            docker_fetching_containers: false,
            docker_fetch_error: None,
            docker_removing: false,
            docker_remove_results: None,
            
            // ... rest of fields ...
        }
    }
}
```

- [ ] **Step 6: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds (may have warnings about unused fields)

- [ ] **Step 7: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): update SshTab state for new Docker dialogs

Replace old validation-based state with two-step wizard state.
Add step tracking, inspection state, and parsed results.
Add Remove Containers dialog state with selection tracking and results.

Remove old docker_metadata, docker_validating, docker_validation_error fields.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Implement Install Dialog Step 1 (Image Inspection)

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:839-1012`

**Interfaces:**
- Consumes: `self.docker_install_step`, ViewModel `inspect_docker_image()` method
- Produces: Renders Step 1 UI, handles image input and inspection trigger

- [ ] **Step 1: Locate render_docker_install_dialog function**

Find `fn render_docker_install_dialog()` around line 839

- [ ] **Step 2: Replace function start with step switch**

Replace the function body start (keep signature) with:

```rust
fn render_docker_install_dialog(
    &mut self,
    ctx: &egui::Context,
    mut vm: Option<&mut crate::viewmodel::ViewModel>,
) {
    use egui_material3::MaterialButton;

    let mut dialog_open = self.show_docker_install_dialog;

    egui::Window::new("Install Docker Image")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .open(&mut dialog_open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(8.0);

                // Render different steps
                match self.docker_install_step {
                    1 => self.render_install_step1(ui, vm.as_deref_mut()),
                    2 => self.render_install_step2(ui, vm.as_deref_mut()),
                    _ => {
                        ui.label("Invalid step");
                    }
                }
            });
        });

    self.show_docker_install_dialog = dialog_open;
}
```

- [ ] **Step 3: Add render_install_step1 method**

Add after `render_docker_install_dialog`:

```rust
fn render_install_step1(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
    use egui_material3::MaterialButton;

    ui.label(egui::RichText::new("Step 1: Image Inspection").strong());
    ui.add_space(8.0);

    // Image input
    ui.horizontal(|ui| {
        ui.label("Image:");
        ui.text_edit_singleline(&mut self.docker_image_input);
    });
    ui.label("Format: owner/image or owner/image:tag (default: latest)");
    ui.add_space(8.0);

    // Inspection status
    if self.docker_inspecting {
        ui.spinner();
        ui.label("Pulling and inspecting image...");
        ui.label("This may take 10-60 seconds depending on image size.");
    } else if let Some(error) = &self.docker_inspect_error {
        ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
    }
    ui.add_space(16.0);

    // Action buttons
    ui.horizontal(|ui| {
        let can_inspect = !self.docker_image_input.is_empty() && !self.docker_inspecting;

        if ui.add_enabled(can_inspect, MaterialButton::filled("Inspect Image")).clicked() {
            self.start_image_inspection(vm.as_deref_mut());
        }

        if ui.add(MaterialButton::text("Cancel")).clicked() {
            self.show_docker_install_dialog = false;
            self.reset_install_dialog_state();
        }
    });
}
```

- [ ] **Step 4: Add start_image_inspection helper**

Add after `render_install_step1`:

```rust
fn start_image_inspection(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
    // Parse image:tag
    let full_input = self.docker_image_input.trim();
    let (image, tag) = if let Some((img, tg)) = full_input.rsplit_once(':') {
        (img.to_string(), tg.to_string())
    } else {
        (full_input.to_string(), "latest".to_string())
    };

    if let Some(host_idx) = self.docker_install_host_idx {
        if let Some(row) = self.rows.get(host_idx) {
            if let Some(ref mut vm) = vm {
                self.docker_inspecting = true;
                self.docker_inspect_error = None;
                self.docker_parsed_image = image.clone();
                self.docker_parsed_tag = tag.clone();

                eprintln!("🔍 UI: Starting image inspection for {}:{}", image, tag);
                let _ = vm.inspect_docker_image(row.host.clone(), image, tag);
            }
        }
    }
}
```

- [ ] **Step 5: Add reset_install_dialog_state helper**

Add after `start_image_inspection`:

```rust
fn reset_install_dialog_state(&mut self) {
    self.docker_install_step = 1;
    self.docker_image_input.clear();
    self.docker_inspecting = false;
    self.docker_inspect_error = None;
    self.docker_container_name.clear();
    self.docker_parsed_image.clear();
    self.docker_parsed_tag.clear();
    self.docker_exposed_ports.clear();
    self.docker_env_vars.clear();
    self.docker_port_mappings.clear();
    self.docker_env_overrides.clear();
    self.docker_installing = false;
    self.docker_install_success = false;
    self.docker_install_error = None;
}
```

- [ ] **Step 6: Add render_install_step2 stub**

Add after `reset_install_dialog_state`:

```rust
fn render_install_step2(&mut self, ui: &mut egui::Ui, _vm: Option<&mut crate::viewmodel::ViewModel>) {
    ui.label("Step 2 placeholder - will be implemented in next task");
}
```

- [ ] **Step 7: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 8: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement Install Dialog Step 1 (image inspection)

Replace old validation-based dialog with two-step wizard structure.
Step 1 allows image input and triggers inspection via inspect_docker_image().
Parse image:tag (default to latest) and pass to ViewModel.

Add helpers: start_image_inspection(), reset_install_dialog_state().
Add stub for Step 2 (to be implemented next).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Implement Install Dialog Step 2 (Configuration & Install)

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:1-end`

**Interfaces:**
- Consumes: `self.docker_exposed_ports`, `self.docker_env_vars`, ViewModel `install_docker_image()`
- Produces: Renders Step 2 UI with pre-filled port/env configuration, handles install

- [ ] **Step 1: Replace render_install_step2 stub**

Find `render_install_step2` and replace with:

```rust
fn render_install_step2(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
    use egui_material3::MaterialButton;

    ui.label(egui::RichText::new("Step 2: Configuration").strong());
    ui.add_space(8.0);

    // Show image info
    ui.label(format!("Image: {}:{}", self.docker_parsed_image, self.docker_parsed_tag));
    ui.add_space(8.0);

    // Container name
    ui.horizontal(|ui| {
        ui.label("Container Name:");
        ui.text_edit_singleline(&mut self.docker_container_name);
    });
    ui.add_space(8.0);

    // Port mappings
    ui.label(egui::RichText::new("Port Mappings:").strong());
    ui.add_space(4.0);

    let mut to_remove = None;
    for (idx, (host_port, container_port)) in self.docker_port_mappings.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label("Host:");
            ui.add(egui::TextEdit::singleline(host_port).desired_width(80.0));
            ui.label("→ Container:");
            ui.add(egui::TextEdit::singleline(container_port).desired_width(80.0));
            if ui.button("−").clicked() {
                to_remove = Some(idx);
            }
        });
    }
    if let Some(idx) = to_remove {
        self.docker_port_mappings.remove(idx);
    }

    if ui.add(MaterialButton::text("+ Add Port Mapping")).clicked() {
        self.docker_port_mappings.push(("".to_string(), "".to_string()));
    }
    ui.add_space(8.0);

    // Environment variables
    ui.label(egui::RichText::new("Environment Variables:").strong());
    ui.add_space(4.0);

    let mut to_remove_env = None;
    for (idx, (key, value)) in self.docker_env_overrides.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(key).desired_width(150.0).hint_text("KEY"));
            ui.label("=");
            ui.add(egui::TextEdit::singleline(value).desired_width(200.0).hint_text("value"));
            if ui.button("−").clicked() {
                to_remove_env = Some(idx);
            }
        });
    }
    if let Some(idx) = to_remove_env {
        self.docker_env_overrides.remove(idx);
    }

    if ui.add(MaterialButton::text("+ Add Environment Variable")).clicked() {
        self.docker_env_overrides.push(("".to_string(), "".to_string()));
    }
    ui.add_space(16.0);

    // Installation status
    if self.docker_installing {
        ui.spinner();
        ui.label("Installing container...");
    } else if self.docker_install_success {
        ui.colored_label(egui::Color32::GREEN, "✓ Container installed successfully");
    } else if let Some(error) = &self.docker_install_error {
        ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
    }
    ui.add_space(8.0);

    // Action buttons
    ui.horizontal(|ui| {
        // Back button (not shown if install succeeded)
        if !self.docker_install_success {
            if ui.add(MaterialButton::text("Back")).clicked() {
                self.docker_install_step = 1;
                self.docker_inspecting = false;
                self.docker_inspect_error = None;
            }
        }

        // Install or Close button
        if self.docker_install_success {
            if ui.add(MaterialButton::filled("Close")).clicked() {
                self.show_docker_install_dialog = false;
                self.reset_install_dialog_state();
            }
        } else {
            let can_install = !self.docker_container_name.is_empty() && !self.docker_installing;

            if ui.add_enabled(can_install, MaterialButton::filled("Install")).clicked() {
                self.start_container_installation(vm.as_deref_mut());
            }

            if ui.add(MaterialButton::text("Cancel")).clicked() {
                self.show_docker_install_dialog = false;
                self.reset_install_dialog_state();
            }
        }
    });
}
```

- [ ] **Step 2: Add start_container_installation helper**

Add after `render_install_step2`:

```rust
fn start_container_installation(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
    if let Some(host_idx) = self.docker_install_host_idx {
        if let Some(row) = self.rows.get(host_idx) {
            if let Some(ref mut vm) = vm {
                // Parse port mappings
                let ports: Vec<(u16, u16)> = self.docker_port_mappings
                    .iter()
                    .filter_map(|(h, c)| {
                        let host = h.parse::<u16>().ok()?;
                        let container = c.parse::<u16>().ok()?;
                        Some((host, container))
                    })
                    .collect();

                // Filter out empty env vars
                let env: Vec<(String, String)> = self.docker_env_overrides
                    .iter()
                    .filter(|(k, _)| !k.is_empty())
                    .cloned()
                    .collect();

                self.docker_installing = true;
                self.docker_install_error = None;

                eprintln!("🔍 UI: Starting container installation");
                eprintln!("  Container: {}", self.docker_container_name);
                eprintln!("  Image: {}:{}", self.docker_parsed_image, self.docker_parsed_tag);

                let _ = vm.install_docker_image(
                    row.host.clone(),
                    self.docker_container_name.clone(),
                    self.docker_parsed_image.clone(),
                    self.docker_parsed_tag.clone(),
                    ports,
                    env,
                );
            }
        }
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 4: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement Install Dialog Step 2 (configuration + install)

Show pre-filled port mappings and env vars from inspection.
Allow user to edit, add, or remove mappings before install.
Back button returns to Step 1, Install button triggers installation.
Success state shows Close button only.

Add helper: start_container_installation().

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Add Container Name Generation Helper

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:1-end`

**Interfaces:**
- Consumes: Image name, existing containers list
- Produces: `generate_container_name(image: &str, existing: &[DockerContainerConfig]) -> String`

- [ ] **Step 1: Add helper function**

Add at end of file before closing brace:

```rust
/// Generate unique container name from image
/// Example: "linuxserver/wireguard:latest" → "wireguard-1"
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

- [ ] **Step 2: Add unit tests**

Add after function:

```rust
#[cfg(test)]
mod docker_name_tests {
    use super::*;
    use crate::config::DockerContainerConfig;

    fn make_container(name: &str) -> DockerContainerConfig {
        DockerContainerConfig {
            name: name.to_string(),
            image: "test".to_string(),
            tag: "latest".to_string(),
            ports: vec![],
            env: vec![],
            status: "running".to_string(),
        }
    }

    #[test]
    fn test_generate_container_name_simple() {
        let containers = vec![];
        let name = generate_container_name("nginx", &containers);
        assert_eq!(name, "nginx-1");
    }

    #[test]
    fn test_generate_container_name_with_owner() {
        let containers = vec![];
        let name = generate_container_name("linuxserver/wireguard", &containers);
        assert_eq!(name, "wireguard-1");
    }

    #[test]
    fn test_generate_container_name_with_tag() {
        let containers = vec![];
        let name = generate_container_name("redis:7", &containers);
        assert_eq!(name, "redis-1");
    }

    #[test]
    fn test_generate_container_name_increments() {
        let containers = vec![
            make_container("nginx-1"),
            make_container("nginx-2"),
        ];
        let name = generate_container_name("nginx", &containers);
        assert_eq!(name, "nginx-3");
    }

    #[test]
    fn test_generate_container_name_full_format() {
        let containers = vec![make_container("wireguard-1")];
        let name = generate_container_name("linuxserver/wireguard:latest", &containers);
        assert_eq!(name, "wireguard-2");
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib -p dure docker_name_tests`
Expected: All 5 tests PASS

- [ ] **Step 4: Call from start_image_inspection**

Find `start_image_inspection` and add after inspection starts:

```rust
// In start_image_inspection, after setting docker_parsed_image/tag
self.docker_container_name = generate_container_name(
    &self.docker_parsed_image,
    &self.rows.get(host_idx).map(|r| &r.docker_containers).unwrap_or(&vec![]),
);
```

Wait - we need to access row.docker_containers, so update the code:

```rust
fn start_image_inspection(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
    // Parse image:tag
    let full_input = self.docker_image_input.trim();
    let (image, tag) = if let Some((img, tg)) = full_input.rsplit_once(':') {
        (img.to_string(), tg.to_string())
    } else {
        (full_input.to_string(), "latest".to_string())
    };

    if let Some(host_idx) = self.docker_install_host_idx {
        if let Some(row) = self.rows.get(host_idx) {
            if let Some(ref mut vm) = vm {
                self.docker_inspecting = true;
                self.docker_inspect_error = None;
                self.docker_parsed_image = image.clone();
                self.docker_parsed_tag = tag.clone();
                
                // Generate container name
                self.docker_container_name = generate_container_name(&image, &row.docker_containers);

                eprintln!("🔍 UI: Starting image inspection for {}:{}", image, tag);
                let _ = vm.inspect_docker_image(row.host.clone(), image, tag);
            }
        }
    }
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 6: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add container name generation helper

Auto-generate unique container names from image name.
Extract base name (remove owner, tag) and increment counter.

Tests cover: simple name, owner/image, tag, incrementing, full format.

Integrate into start_image_inspection() to auto-fill container name.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 10: Implement Remove Containers Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:1-end`

**Interfaces:**
- Consumes: `self.docker_available_containers`, ViewModel `list_docker_containers()`, `remove_docker_containers()`
- Produces: Renders removal dialog with checkboxes, handles batch deletion

- [ ] **Step 1: Add render_docker_remove_dialog method**

Add after `render_docker_install_dialog`:

```rust
fn render_docker_remove_dialog(
    &mut self,
    ctx: &egui::Context,
    mut vm: Option<&mut crate::viewmodel::ViewModel>,
) {
    use egui_material3::MaterialButton;

    let mut dialog_open = self.show_docker_remove_dialog;

    egui::Window::new("Remove Docker Containers")
        .collapsible(false)
        .resizable(true)
        .default_width(600.0)
        .open(&mut dialog_open)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(8.0);

                // Loading state
                if self.docker_fetching_containers {
                    ui.spinner();
                    ui.label("Loading containers...");
                    return;
                }

                // Error state
                if let Some(error) = &self.docker_fetch_error {
                    ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
                    ui.add_space(8.0);
                    if ui.add(MaterialButton::filled("Retry")).clicked() {
                        self.load_containers_for_removal(vm.as_deref_mut());
                    }
                    if ui.add(MaterialButton::text("Close")).clicked() {
                        self.show_docker_remove_dialog = false;
                        self.reset_remove_dialog_state();
                    }
                    return;
                }

                // Results display
                if let Some(ref results) = self.docker_remove_results {
                    self.render_removal_results(ui, results);
                    ui.add_space(8.0);
                    if ui.add(MaterialButton::filled("Close")).clicked() {
                        self.show_docker_remove_dialog = false;
                        self.reset_remove_dialog_state();
                    }
                    return;
                }

                // Container selection
                if self.docker_available_containers.is_empty() {
                    ui.label("No containers found on this host.");
                    ui.add_space(8.0);
                    if ui.add(MaterialButton::text("Close")).clicked() {
                        self.show_docker_remove_dialog = false;
                        self.reset_remove_dialog_state();
                    }
                    return;
                }

                self.render_container_selection(ui);

                ui.add_space(16.0);

                // Action buttons
                self.render_removal_actions(ui, vm.as_deref_mut());
            });
        });

    self.show_docker_remove_dialog = dialog_open;
}
```

- [ ] **Step 2: Add render_container_selection helper**

Add after `render_docker_remove_dialog`:

```rust
fn render_container_selection(&mut self, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Select containers to remove:").strong());
    ui.add_space(8.0);

    for container in &self.docker_available_containers {
        let is_selected = self.docker_selected_containers.contains(&container.name);

        ui.horizontal(|ui| {
            let mut selected = is_selected;
            if ui.checkbox(&mut selected, "").changed() {
                if selected {
                    self.docker_selected_containers.push(container.name.clone());
                } else {
                    self.docker_selected_containers.retain(|n| n != &container.name);
                }
            }

            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&container.name).strong());
                ui.label(format!("Image: {}:{}", container.image, container.tag));
                ui.label(format!("Status: {}", container.status));
                if !container.ports.is_empty() {
                    let ports: Vec<String> = container.ports.iter()
                        .map(|(h, c)| format!("{}→{}", h, c))
                        .collect();
                    ui.label(format!("Ports: {}", ports.join(", ")));
                }
            });
        });

        ui.add_space(4.0);
    }

    ui.add_space(8.0);
    ui.label(format!(
        "{} of {} containers selected",
        self.docker_selected_containers.len(),
        self.docker_available_containers.len()
    ));
}
```

- [ ] **Step 3: Add render_removal_actions helper**

Add after `render_container_selection`:

```rust
fn render_removal_actions(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
    use egui_material3::MaterialButton;

    ui.horizontal(|ui| {
        // Select/Deselect All buttons
        if ui.add(MaterialButton::text("Select All")).clicked() {
            self.docker_selected_containers = self.docker_available_containers
                .iter()
                .map(|c| c.name.clone())
                .collect();
        }

        if ui.add(MaterialButton::text("Deselect All")).clicked() {
            self.docker_selected_containers.clear();
        }
    });

    ui.add_space(8.0);

    // Deletion section
    if self.docker_removing {
        ui.spinner();
        ui.label(format!(
            "Removing {} containers...",
            self.docker_selected_containers.len()
        ));
    } else {
        ui.horizontal(|ui| {
            let can_delete = !self.docker_selected_containers.is_empty();

            if ui.add_enabled(can_delete, MaterialButton::filled("Delete Selected"))
                .on_hover_text(if !can_delete {
                    "Select containers to remove"
                } else {
                    "Remove selected containers (cannot be undone)"
                })
                .clicked()
            {
                self.confirm_removal(ui);
            }

            if ui.add(MaterialButton::text("Cancel")).clicked() {
                self.show_docker_remove_dialog = false;
                self.reset_remove_dialog_state();
            }
        });

        // Inline confirmation
        if ui.data(|d| d.get_temp::<bool>(egui::Id::new("docker_removal_confirm"))).unwrap_or(false) {
            ui.add_space(8.0);
            ui.colored_label(
                egui::Color32::from_rgb(200, 100, 0),
                format!("Remove {} containers? This cannot be undone.", self.docker_selected_containers.len())
            );
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::filled("Confirm")).clicked() {
                    ui.data_mut(|d| d.remove::<bool>(egui::Id::new("docker_removal_confirm")));
                    self.start_container_removal(vm.as_deref_mut());
                }
                if ui.add(MaterialButton::text("Cancel")).clicked() {
                    ui.data_mut(|d| d.remove::<bool>(egui::Id::new("docker_removal_confirm")));
                }
            });
        }
    }
}
```

- [ ] **Step 4: Add render_removal_results helper**

Add after `render_removal_actions`:

```rust
fn render_removal_results(&self, ui: &mut egui::Ui, results: &RemoveResults) {
    ui.label(egui::RichText::new("Removal Results:").strong());
    ui.add_space(8.0);

    if !results.removed.is_empty() {
        ui.colored_label(
            egui::Color32::GREEN,
            format!("✓ Removed: {}", results.removed.join(", "))
        );
    }

    if !results.failed.is_empty() {
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Failed:").color(egui::Color32::RED));
        for (name, error) in &results.failed {
            ui.colored_label(
                egui::Color32::RED,
                format!("  ⚠ {}: {}", name, error)
            );
        }
    }

    if results.removed.is_empty() && results.failed.is_empty() {
        ui.label("No containers were processed.");
    }
}
```

- [ ] **Step 5: Add dialog lifecycle helpers**

Add after `render_removal_results`:

```rust
fn load_containers_for_removal(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
    if let Some(host_idx) = self.docker_remove_host_idx {
        if let Some(row) = self.rows.get(host_idx) {
            self.docker_fetching_containers = true;
            self.docker_fetch_error = None;
            
            // Load from config (already available)
            self.docker_available_containers = row.docker_containers.clone();
            self.docker_fetching_containers = false;
            
            eprintln!("🔍 UI: Loaded {} containers for removal", self.docker_available_containers.len());
        }
    }
}

fn confirm_removal(&self, ui: &mut egui::Ui) {
    ui.data_mut(|d| d.insert_temp(egui::Id::new("docker_removal_confirm"), true));
}

fn start_container_removal(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
    if let Some(host_idx) = self.docker_remove_host_idx {
        if let Some(row) = self.rows.get(host_idx) {
            if let Some(ref mut vm) = vm {
                self.docker_removing = true;
                
                eprintln!("🔍 UI: Starting container removal");
                eprintln!("  Containers: {:?}", self.docker_selected_containers);
                
                let _ = vm.remove_docker_containers(
                    row.host.clone(),
                    self.docker_selected_containers.clone(),
                );
            }
        }
    }
}

fn reset_remove_dialog_state(&mut self) {
    self.docker_available_containers.clear();
    self.docker_selected_containers.clear();
    self.docker_fetching_containers = false;
    self.docker_fetch_error = None;
    self.docker_removing = false;
    self.docker_remove_results = None;
}
```

- [ ] **Step 6: Trigger dialog on button click**

Find `process_action_triggers` method and add after Docker install trigger:

```rust
// Docker remove containers trigger
let remove_id = egui::Id::new(format!("ssh_remove_containers_{}", idx));
if let Some(host) = ui.data(|d| d.get_temp::<String>(remove_id)) {
    ui.data_mut(|d| d.remove::<String>(remove_id));
    
    self.show_docker_remove_dialog = true;
    self.docker_remove_host_idx = Some(idx);
    self.load_containers_for_removal(vm.as_deref_mut());
}
```

- [ ] **Step 7: Call dialog renderer in ui() method**

Find the dialog rendering section in `ui()` and add:

```rust
if self.show_docker_remove_dialog {
    self.render_docker_remove_dialog(ui.ctx(), vm.as_deref_mut());
}
```

- [ ] **Step 8: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 9: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement Remove Containers dialog

Multi-select container list with checkboxes.
Batch removal with confirmation prompt.
Results display showing successes and failures separately.

Helpers: render_container_selection, render_removal_actions,
render_removal_results, load_containers_for_removal,
confirm_removal, start_container_removal, reset_remove_dialog_state.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 11: Replace Uninstall Docker Button

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:2114-2300`

**Interfaces:**
- Consumes: `row.docker_enabled`, `row.docker_containers`
- Produces: "Remove Containers" button in operations row

- [ ] **Step 1: Locate render_operations function**

Find `fn render_operations()` around line 2114

- [ ] **Step 2: Find Uninstall Docker button**

Search for `"Uninstall Docker"` around line 2190

- [ ] **Step 3: Replace with Remove Containers button**

Replace:

```rust
if ui
    .add(MaterialButton::outlined("Uninstall Docker").small())
    .on_hover_text("Uninstall Docker")
    .clicked()
{
    ui.data_mut(|d| {
        d.insert_temp(
            egui::Id::new(format!("ssh_uninstall_docker_{}", idx)),
            row.host.clone(),
        )
    });
}
```

With:

```rust
// Only show if Docker is enabled AND has containers
if !row.docker_containers.is_empty() {
    if ui
        .add(MaterialButton::outlined("Remove Containers").small())
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

- [ ] **Step 4: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 5: Test button placement**

Visual check: Button appears only when Docker is enabled and containers exist

- [ ] **Step 6: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): replace Uninstall Docker with Remove Containers button

Remove 'Uninstall Docker' button from operations row.
Add 'Remove Containers' button (shown only when containers exist).
Triggers docker_remove_dialog when clicked.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 12: Add Event Handlers for New Events

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:335-600`

**Interfaces:**
- Consumes: `DockerImageInspected`, `DockerContainersRemoved` events
- Produces: Updates dialog state based on events

- [ ] **Step 1: Locate handle_event method**

Find `fn handle_event()` around line 335

- [ ] **Step 2: Add DockerImageInspected handler**

Add after `DockerStatusRetrieved` handler:

```rust
ViewModelEvent::Ssh(SshEvent::DockerImageInspected {
    image,
    tag,
    exposed_ports,
    env_vars,
}) => {
    eprintln!("✓ Docker image inspected: {}:{}", image, tag);
    eprintln!("  Ports: {:?}", exposed_ports);
    eprintln!("  Env vars: {} variables", env_vars.len());

    // Update dialog state
    self.docker_inspecting = false;
    self.docker_inspect_error = None;
    self.docker_exposed_ports = exposed_ports.clone();
    self.docker_env_vars = env_vars.clone();

    // Pre-fill port mappings (host=container)
    self.docker_port_mappings = exposed_ports
        .iter()
        .map(|&port| (port.to_string(), port.to_string()))
        .collect();

    // Pre-fill env vars (editable copy)
    self.docker_env_overrides = env_vars;

    // Advance to step 2
    self.docker_install_step = 2;
}
```

- [ ] **Step 3: Add DockerContainersRemoved handler**

Add after `DockerContainerRemoved` handler:

```rust
ViewModelEvent::Ssh(SshEvent::DockerContainersRemoved {
    host_name,
    removed,
    failed,
}) => {
    eprintln!("✓ Docker containers removal complete for {}", host_name);
    eprintln!("  Removed: {} containers", removed.len());
    eprintln!("  Failed: {} containers", failed.len());

    // Update dialog state
    self.docker_removing = false;
    self.docker_remove_results = Some(RemoveResults {
        removed: removed.clone(),
        failed: failed.clone(),
    });

    // Update config - remove successfully removed containers
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok((mut app_config, config_path)) = load_config() {
        if let Some(host_config) = app_config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
            host_config.docker_containers.retain(|c| !removed.contains(&c.name));
            let _ = app_config.save(&config_path);
        }
    }

    // Update row data
    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host_name) {
        row.docker_containers.retain(|c| !removed.contains(&c.name));
    }
}
```

- [ ] **Step 4: Update Error handler for inspection failures**

Find the `SshEvent::Error` handler and add:

```rust
// In Error handler, add check for inspection operations
if operation.contains("inspect_docker_image") {
    self.docker_inspecting = false;
    self.docker_inspect_error = Some(error.clone());
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds

- [ ] **Step 6: Commit changes**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add event handlers for new Docker events

Handle DockerImageInspected:
- Pre-fill port mappings and env vars
- Advance to step 2 of wizard

Handle DockerContainersRemoved:
- Update config file (remove deleted containers)
- Update row data for drawer display
- Show results in dialog

Handle inspection errors via Error event.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 13: Remove Old ValidateDockerImage Code

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs:1-end`
- Modify: `mobile/src/viewmodel/ssh/commands.rs:44-46`
- Modify: `mobile/src/viewmodel/ssh/actor.rs:1-end`

**Interfaces:**
- Consumes: Old `ValidateDockerImage` command and handler
- Produces: Clean codebase without unused validation code

- [ ] **Step 1: Remove ValidateDockerImage command variant**

In `mobile/src/viewmodel/ssh/commands.rs`, find and delete:

```rust
ValidateDockerImage {
    image: String,
},
```

- [ ] **Step 2: Remove validate_docker_image ViewModel method**

In `mobile/src/viewmodel/mod.rs`, find and delete the entire method:

```rust
pub fn validate_docker_image(&self, image: String) -> anyhow::Result<()> {
    // ... delete entire method body ...
}
```

- [ ] **Step 3: Remove ValidateDockerImage handler in actor**

In `mobile/src/viewmodel/ssh/actor.rs`, find and delete the handler:

```rust
SshCommand::ValidateDockerImage { image } => {
    // ... delete entire handler ...
}
```

- [ ] **Step 4: Remove DockerImageValidated event (if no longer used)**

In `mobile/src/viewmodel/ssh/events.rs`, check if `DockerImageValidated` is still used elsewhere. If not, delete:

```rust
DockerImageValidated {
    image: String,
    metadata: DockerImageMetadata,
},
```

- [ ] **Step 5: Remove event handler in UI**

In `mobile/src/ui_tabs/ssh.rs`, find `DockerImageValidated` handler in `handle_event` and delete it

- [ ] **Step 6: Verify compilation**

Run: `cargo build --lib -p dure`
Expected: Compilation succeeds with no errors (warnings about unused imports are OK)

- [ ] **Step 7: Clean up imports if needed**

If `DockerImageMetadata` is no longer used, remove the import from `ssh/events.rs`

- [ ] **Step 8: Verify no remaining references**

Run: `grep -r "ValidateDockerImage\|validate_docker_image" mobile/src/`
Expected: No matches (or only in comments/docs)

- [ ] **Step 9: Commit changes**

```bash
git add mobile/src/viewmodel/mod.rs mobile/src/viewmodel/ssh/commands.rs mobile/src/viewmodel/ssh/actor.rs mobile/src/viewmodel/ssh/events.rs mobile/src/ui_tabs/ssh.rs
git commit -m "refactor(ssh): remove old Docker Hub API validation code

Remove ValidateDockerImage command/event/handlers.
No longer needed - inspection now happens on remote host via docker pull/history.

Cleanup: remove validate_docker_image() method, DockerImageValidated event,
and related event handlers.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Testing Checklist

After all tasks are complete, manually verify:

### Install Dialog
- [ ] Enter image without tag (e.g., `nginx`) → defaults to `latest`
- [ ] Enter image with tag (e.g., `redis:7`) → uses specified tag
- [ ] Inspect button triggers docker pull (watch logs)
- [ ] Step 2 shows pre-filled ports from EXPOSE directives
- [ ] Step 2 shows pre-filled env vars from ENV directives
- [ ] Container name auto-generates uniquely (wireguard-1, wireguard-2)
- [ ] Can edit port mappings before install
- [ ] Can add/remove env vars before install
- [ ] Back button returns to Step 1
- [ ] Install button triggers container launch
- [ ] Success shows "Close" button only
- [ ] Errors keep Install button enabled for retry

### Remove Dialog
- [ ] Dialog loads container list from config
- [ ] Checkboxes allow multi-select
- [ ] "Select All" / "Deselect All" work
- [ ] Delete button disabled when nothing selected
- [ ] Confirmation prompt appears
- [ ] Batch removal processes all selected
- [ ] Results show successes and failures
- [ ] Config file updates after removal
- [ ] Drawer updates to remove deleted containers

### Button Replacement
- [ ] "Uninstall Docker" button no longer appears
- [ ] "Remove Containers" button appears when docker_enabled=true and containers exist
- [ ] Button hidden when no containers

### Error Handling
- [ ] Image not found shows error in Step 1
- [ ] Docker pull failure shows error
- [ ] Container removal failure shows in results
- [ ] SSH errors display appropriate messages

---

## Implementation Complete

All tasks implement the approved design spec. The two-step wizard provides automatic port/env detection via `docker pull` + `docker history`, and the removal dialog enables batch container deletion with detailed results.

Total LOC estimate: ~800 lines (new code + modifications)
Total files modified: 5 files
Total commits: 13 commits
