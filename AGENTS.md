# Repository Guidelines

Follow global defaults; this file contains only repository-specific additions.

## Purpose

This repository holds Model Context Protocol (MCP) servers that are reusable across projects and coding agents.
An MCP server belongs here only when it is project-agnostic and functions in any codebase or environment.

## Portability

- No project names, service names, domain rules, or credentials.
- No hard-coded file system paths. The server must accept target paths from tool arguments at runtime.
- No proprietary code or proprietary test fixtures. Benchmarks and tests must use synthetic or anonymized data.
- Favor lightweight, native implementations (e.g. single-binary Rust or zero-dependency scripts) for minimal startup latency and memory footprint.
- All servers communicate via standard JSON-RPC 2.0 over `stdio`.

## Writing Style

- Write documentation, code comments, and commit messages in English.
- Lead with the most important information and drop anything repeated.
- Put each prose sentence on its own source line.
- Explain the architectural motivation and trade-offs before implementation details.

## Repository Structure

- Active servers live at `servers/<category>/<server-name>/`.
- Category directories group related servers (`filesystem`, `code-intelligence`, `database`, etc.).
- Each server must have its own `README.md` documenting tool schemas, configuration examples, and benchmarks.
- Update the root `README.md` catalog whenever a server is added, renamed, or recategorized.

```
mcps/
├── README.md
├── AGENTS.md
├── CLAUDE.md -> AGENTS.md
└── servers/
    └── <category>/
        └── <server-name>/
            ├── README.md
            └── ... (source files)
```

## Testing

- Every server must include automated unit tests (e.g. `cargo test`) or a self-contained verification suite.
- Run tests and check clean formatting before submitting changes.
