# Documentation Index

This directory contains documentation for the Dure distributed e-commerce platform.

## Start Here

**New to the project?** Read these:
1. **[../CLAUDE.md](../CLAUDE.md)** - Complete project guide (start here!)
2. **[../ARCH.sudolang.md](../ARCH.sudolang.md)** - Architecture summary (SudoLang format)
3. **[TROUBLESHOOTING.md](./TROUBLESHOOTING.md)** - Common issues and solutions

## Quick Navigation

### Architecture & Design
- **[../ARCH.sudolang.md](../ARCH.sudolang.md)** - Architecture summary (layers, actors, data flow, tech stack)

### Reference
- **[CLI_REFERENCE.md](./CLI_REFERENCE.md)** - Command-line interface documentation
- **[README.md](./README.md)** - Documentation overview

### Setup Guides
- **[CLOUDFLARE_OAUTH_SETUP.md](./CLOUDFLARE_OAUTH_SETUP.md)** - Cloudflare OAuth configuration
- **[GCP_REST_API_GUIDE.md](./GCP_REST_API_GUIDE.md)** - GCP REST API usage guide
- **[GCP_WIZARD_COMPLETE_FLOW.md](./GCP_WIZARD_COMPLETE_FLOW.md)** - GCP setup wizard flow

### Integration
**WARNING**: May contain beads-specific content:
- **[AGENT_INTEGRATION.md](./AGENT_INTEGRATION.md)** - AI agent integration (needs review)

## Documentation Status

### Available
- **../ARCH.sudolang.md** - Architecture summary (SudoLang format)
- **../CLAUDE.md** - Project guide
- **CLI_REFERENCE.md** - CLI commands
- **TROUBLESHOOTING.md** - Common issues
- **CLOUDFLARE_OAUTH_SETUP.md** - Cloudflare setup
- **GCP_REST_API_GUIDE.md** - GCP API guide
- **GCP_WIZARD_COMPLETE_FLOW.md** - GCP wizard
- **README.md** - Docs overview

### Needs Review
- **AGENT_INTEGRATION.md** - May reference beads commands

### Missing Documentation
- **Testing Guide** - Unit/integration/e2e testing patterns
- **E-commerce Workflows** - Store setup, product management, order processing
- **Payment Integration** - Portone, KakaoPay integration guide
- **Deployment Guides** - Production deployment (GCP, DNS, SSL)
- **API Documentation** - Internal APIs and data models
- **WASM Deployment** - Building and deploying WASM front-end
- **Mobile Build Guide** - Detailed Android build process
- **Security Model** - Identity management, key storage, attestation

## For AI Assistants

When working with this codebase:

1. Read **[../CLAUDE.md](../CLAUDE.md)** first for complete project context
2. Read **[../ARCH.sudolang.md](../ARCH.sudolang.md)** for architecture summary
3. Check this file (INDEX.md) to know which docs are valid
4. Treat documents marked with **WARNING** as unreliable for Dure development
5. Focus on actual code in `mobile/src/` for understanding current architecture

## Contributing to Documentation

- Follow Markdown conventions
- Include code examples where applicable
- Update this index when adding new docs
- Remove beads references when updating docs
- Add real Dure examples and workflows
