# Fastlane Metadata for F-Droid

This directory contains the fastlane metadata structure for F-Droid submission of Dure - a distributed e-commerce platform.

## Directory Structure

```
fastlane/
└── metadata/
    └── android/
        └── en-US/                              # Required fallback locale
            ├── title.txt                       # App name: "Dure"
            ├── short_description.txt           # Brief description (max 80 chars)
            ├── full_description.txt            # Detailed description (max 4000 chars, HTML allowed)
            ├── images/
            │   ├── icon.png                    # App icon (512x512px)
            │   ├── phoneScreenshots/           # Phone screenshots
            │   │   ├── 1.png                   # First screenshot
            │   │   ├── mobile1.png             # Mobile screenshot 1
            │   │   ├── mobile2.png             # Mobile screenshot 2
            │   │   ├── mobile3.png             # Mobile screenshot 3
            │   │   ├── mobile4.png             # Mobile screenshot 4
            │   │   └── mobile5.png             # Mobile screenshot 5
            │   └── README.md                   # Instructions for graphics
            └── changelogs/                     # Version-specific changelogs
                └── 1.txt                       # Changelog for versionCode 1
```

## Files Included

### Required Files ✅
- **title.txt**: "Dure"
- **short_description.txt**: Distributed e-commerce platform description
- **full_description.txt**: Comprehensive app description with features, hosting options, and supported platforms

### Recommended Files ✅
- **images/icon.png**: 512x512px app icon
- **images/phoneScreenshots/**: Multiple mobile screenshots showcasing e-commerce features

### Optional Files ✅
- **changelogs/1.txt**: Initial release changelog for versionCode 1

## Missing Graphics (Optional)

To enhance the F-Droid listing, consider adding:
- **featureGraphic.png**: 1024x500px or 512x250px landscape banner for app description header
- **promoGraphic.png**: 180x120px promotional graphic (rarely used)
- **tvBanner.png**: 1280x720px TV banner (for Android TV)

## F-Droid Compliance

The Dure project is compliant with F-Droid inclusion policy:

### ✅ License Requirements
- Uses dual MIT/Apache-2.0 licensing (both FLOSS-approved)
- All dependencies are FLOSS (egui, eframe, tokio, etc.)

### ✅ Build Requirements
- No Google Play Services dependencies (optional Firebase/Supabase are user-configured backends)
- No proprietary analytics libraries
- Uses standard build tools (Rust/Cargo, Gradle)
- Built with egui framework for cross-platform UI

### ✅ Privacy & Security
- No user tracking or telemetry
- No auto-updates bypassing F-Droid
- Cryptographic identity management (private/public keys)
- User controls their own hosting infrastructure
- Proper application ID: pe.nikescar.dure

### ✅ Source Code
- Publicly available on GitHub: https://github.com/dure-one/dure-installer
- Maintained and up-to-date
- Clear version tags recommended for releases

## App Description Summary

Dure is a distributed e-commerce platform enabling small shop owners to run e-commerce operations without centralized servers:

**Core Features:**
- Identity Management (cryptographic keys, attestation)
- Guest Front & Store Front (WASM-based)
- Hosting Management (DNS, Database, Site deployment)
- Store Management (Products, Orders, Shipments, Accounts)
- Payment Integration (Portone, KakaoPay)

**Hosting Options:**
- Firebase + Firestore
- Supabase
- Google Compute Engine (GCE)
- Cafe24 VPS

**Three Modes:**
- Server Mode (--serv): WebSocket server
- GUI Mode (--tray): Material3 design interface
- CLI Mode: Command-line automation

## Updating for New Releases

1. **Before creating a release tag:**
   - Add new changelog file: `changelogs/<versionCode>.txt`
   - Update screenshots if UI changed significantly
   - Review and update feature descriptions
   - Commit fastlane changes

2. **Version Code Mapping:**
   - Changelog files must be named exactly with the versionCode
   - Example: versionCode=2 → `changelogs/2.txt`
   - Max 500 bytes, plain text (no HTML)

3. **F-Droid picks up metadata from the same tag used to build the APK**

## Resources

- [F-Droid Fastlane Documentation](https://f-droid.org/docs/All_About_Descriptions_Graphics_and_Screenshots/#in-the-apps-source-repository)
- [F-Droid Inclusion Policy](https://f-droid.org/docs/Inclusion_Policy)
- [Fastlane Metadata Structure](https://gitlab.com/-/snippets/1895688)
- [Dure Project Documentation](../../CLAUDE.md)
