---
title: "Tic CLI"
description: "A fast native command-line tool and agent interface for the TIC (tic.io) business intelligence and datasets API (company, person, vehicle, and bankruptcy lookup). Built in Rust for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Tic-Cli/index.md"
---

# Tic CLI

A fast native command-line tool and agent interface for the TIC (tic.io) business intelligence and datasets API (company, person, vehicle, and bankruptcy lookup). Built in Rust Edition 2024 for developers and autonomous AI agents.

## Quickstart

```bash
# Authenticate interactively via browser token flow
tic login

# Or pass key directly via environment
export TIC_API_KEY="your-api-key"

# Query Swedish company by organization number
tic company get 556792-6687
```

## Features

- **Blazing Fast Native Rust**: Sub-millisecond startup times with zero runtime dependencies.
- **AI Agent Native**: Structured JSON output (`--json`) and standardized error envelopes.
- **Secure Keystore Integration**: Token storage in native macOS Keychain, Windows DPAPI, and Linux Secret Service.
- **Multi-Account Workspaces**: Isolate staging, production, and client accounts safely.
- **Complete Intelligence Suite**: Company datasets, beneficial owners, corporate trees, graphs, debtor records, vehicles, and bankruptcies.

## When to Use This CLI

Use the `tic` CLI whenever you need to:
- Inspect Swedish company records and registry filings.
- Search corporate databases by name, address, phone, or email.
- Query beneficial ownership (UBO) networks and corporate hierarchy trees.
- Check credit and Kronofogden debtor summaries.
- Search vehicle registries and track company fleet assets.
- Look up person roles across Swedish business entities.
- Query Swedish bankruptcy filings.
- Automate corporate due diligence using LLMs or autonomous agents.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Tic-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Tic-Cli/llms-full.txt)
- [Pricing](https://spacecorps.github.io/Tic-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/Tic-Cli/auth.md)
- [GitHub Repository](https://github.com/SpaceCorps/Tic-Cli)
