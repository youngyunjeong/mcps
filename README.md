# MCPs

Reusable, high-performance Model Context Protocol (MCP) servers for AI coding agents.
Works with any agent harness that supports the standard MCP stdio protocol: Google Antigravity (AGY), Claude Code, Claude Desktop, Cursor, Codex, Windsurf, and custom harnesses.

Servers here are project-agnostic on purpose.
They carry no environment variables, no service names, and no domain rules from the projects they were extracted from.
All servers prioritize minimal resource usage, zero-cold-start execution, and deterministic behavior.

## Catalog

### Filesystem

Repository path: `servers/filesystem/`

| Server | Language | Use it for |
| --- | --- | --- |
| [`fast-file-editor`](servers/filesystem/fast-file-editor/) | Rust | Ultra-fast file inspection without truncation caps (bypassing 800-line/45KB limits) and atomic unique string replacement (`str_replace`) without line-drift re-read penalties. |

## Quick Start

### 1. Build the Binary

All Rust-based servers compile into single, standalone native binaries with zero external runtime dependencies.

```bash
git clone https://github.com/youngyunjeong/mcps.git
cd mcps/servers/filesystem/fast-file-editor
cargo build --release
cp target/release/fast-file-editor ~/.local/bin/
```

### 2. Configure Your Agent

#### Google Antigravity (AGY)

Add to `~/.gemini/config/mcp_config.json`:

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "fast-file-editor",
      "args": []
    }
  }
}
```

#### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "fast-file-editor",
      "args": []
    }
  }
}
```

#### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "fast-file-editor"
    }
  }
}
```

## Benchmark Highlights

Measured on Apple Silicon (ARM64) comparing traditional agent primitives with `fast-file-editor`:

* **Full File Inspection**: Streams a ~1,200-line file (58.8 KB) in **0.80 ms** in a single call, avoiding 800-line/45KB payload truncation.
* **Distributed Multi-site Editing**: Executes 5 consecutive edits across a 1,000-line file in **2.36 ms** total. Eliminates defensive re-reads completely and saves **~50,000 context tokens** per session.

Detailed benchmark methodology and metrics are available in [`servers/filesystem/fast-file-editor/README.md`](servers/filesystem/fast-file-editor/).

## Structure

```
mcps/
├── README.md               # Catalog and overview
├── AGENTS.md               # Repository maintenance rules for AI agents
├── CLAUDE.md -> AGENTS.md  # Agent guidelines symlink
└── servers/
    └── <category>/
        └── <server-name>/
            ├── README.md   # Server documentation, schemas, and benchmarks
            ├── Cargo.toml  # Package manifest
            └── src/        # Implementation
```

## Writing Servers Here

See [`AGENTS.md`](AGENTS.md) for the maintenance and contribution rules.

## License

MIT
