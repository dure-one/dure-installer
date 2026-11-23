# Fastlane Metadata

This directory contains the Android app-store metadata used for Dure Installer.
It is intentionally documentation-focused: source-of-truth project details live in
[`../CLAUDE.md`](../CLAUDE.md), [`../README.md`](../README.md), and
[`../SUDOLANG.md`](../SUDOLANG.md).

## Current Scope

Dure Installer is a Rust/egui application for small shop owners to deploy and manage
the Mycart and Prosody backend infrastructure. Its main responsibilities are GCP VM
management, DNS, SSH, security hardening, and remote store operations. The customer
shopping client is the separate `dure-sijang` project and is not part of this app.

## Directory Structure

```text
fastlane/
└── metadata/
    └── android/
        └── en-US/
            ├── title.txt
            ├── short_description.txt
            ├── full_description.txt
            ├── changelogs/
            │   ├── 1.txt
            │   └── 8.txt
            └── images/
                ├── README.md
                ├── icon.png
                ├── logo.svg
                ├── logo110.png
                └── logo1024.png
```

There are currently no `phoneScreenshots/` or feature graphics in the repository.
Add them only when they depict the current installer UI and workflows.

## Metadata Files

- `title.txt`: user-facing app name.
- `short_description.txt`: concise installer summary, kept within the 80-character limit.
- `full_description.txt`: current capabilities, supported platforms, and technology.
- `changelogs/<versionCode>.txt`: release notes for the exact Android `versionCode`.
- `images/icon.png`: 512×512 app icon.
- `images/logo*.png` and `images/logo.svg`: source/auxiliary branding assets; not all are
  standard Fastlane listing images.

## Android App Facts

- Application ID: `app.dure.installer`
- UI: native Android client built with Rust, egui, and eframe
- GUI/runtime stack: egui 0.33, Material3 styling, smol, russh, ureq, and rustls
- Runtime access: network access is required for GCP, DNS, SSH, and Mycart operations
- Desktop counterparts support Linux, macOS, and Windows; WASM is reserved for guest/store-front use
- No Google Play Services are required by the app source

## Release Checklist

1. Update the description if the installer capabilities or product boundary change.
2. Add or update `changelogs/<versionCode>.txt` using the generated Android version code.
3. Keep each changelog at or below 500 bytes and use plain text.
4. Add current phone screenshots when available; remove stale screenshots rather than
   showing unsupported customer-shopping features.
5. Verify `icon.png` remains 512×512.
6. Review the Android app ID and version values in the Gradle build configuration.
7. Commit metadata with the matching release tag so store tooling reads consistent data.

## Build and Technology Notes

The Android build is driven by `mobile/build.sh`: it compiles the Rust shared library
with `cargo-ndk`, packages it with Gradle, and supports multiple Android ABIs in CI.
This project requires Rust nightly and uses a pure-Rust networking/security stack
without OpenSSL. Metadata does not establish F-Droid inclusion compliance by itself;
that must be verified against the tagged source and the applicable F-Droid recipe.

## Resources

- [F-Droid Fastlane documentation](https://f-droid.org/docs/All_About_Descriptions_Graphics_and_Screenshots/#in-the-apps-source-repository)
- [F-Droid inclusion policy](https://f-droid.org/docs/Inclusion_Policy)
- [Project README](../README.md)
- [Project architecture](../SUDOLANG.md)
