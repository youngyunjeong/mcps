# fast-file-editor

High-performance Model Context Protocol (MCP) server written in Rust for fast file inspection and atomic unique string replacement.
Designed for AI coding agents (Antigravity/AGY, Claude Code, Cursor, Codex, Windsurf) to eliminate line-drift re-reading penalties and payload truncation caps.

## The Problem

Most coding agent harnesses provide two built-in primitives:
1. **Line-range file viewing**: Often hard-capped (e.g. 800 lines or 45KB per call), forcing multiple round-trips to read large files.
2. **Line-indexed editing**: Demands strict `start_line` and `end_line` parameters.
   When an agent makes multiple edits across a large file, the first edit shifts the line numbers of all subsequent sections (*line drift*).
   To avoid range search errors, the agent is forced to re-read the file between edits, blowing up token consumption and turn latency.

`fast-file-editor` provides two native tools to solve both problems:
- `view_file_fast`: Streams file contents with line numbers in a single sub-millisecond call, without 800-line or 45KB truncation limits.
- `str_replace`: Performs atomic string replacement based on unique substring matching (Claude Code style).
  No line numbers required; edits never suffer from line drift.

## Tools

### 1. `str_replace`

Replaces an exact unique substring `old_str` with `new_str` in a target file.

- **Uniqueness Guarantee**:
  - If `old_str` occurs 0 times: returns an explicit error indicating exact match failure.
  - If `old_str` occurs >1 times: returns an error showing the match count and asking the agent to include more surrounding context lines.
  - If `old_str` occurs exactly 1 time: atomically replaces the text and returns the line number where the edit took place.
- **Atomic File Write**: Writes replacement to a temporary file in the same directory before performing an atomic rename, preventing file corruption.

```json
{
  "path": "/absolute/path/to/file.ts",
  "old_str": "const isPlayEnded = play.status === \"ENDED\";",
  "new_str": "const isPlayEnded = play.status === \"ENDED\";\nconst hasActiveSeries = Boolean(activeSeries);"
}
```

### 2. `view_file_fast`

Fast file viewer that outputs 1-indexed line numbers.

- **No Truncation Cap**: Returns the requested range or the entire file in a single response without hitting 800-line or 45KB limits.
- **Parameters**:
  - `path` (string, required): Absolute file path.
  - `start_line` (integer, optional, default: 1): Starting line number (1-indexed).
  - `limit` (integer, optional, default: 2000): Maximum lines to return.

## Benchmark

Tested on Apple Silicon (ARM64) using a release build (`cargo build --release`, 393 KB binary).

### 1. Viewing a Large Production File (~1,200 lines, 58.8 KB)

| Metric | Traditional Agent Tool (`view_file`) | `fast-file-editor` (`view_file_fast`) | Improvement |
| :--- | :--- | :--- | :--- |
| **Execution Latency** | ~5 - 15 ms | **0.80 ms** | **10x faster** |
| **Completeness** | Truncated (800 lines / 45KB cap) | **100% complete (1,229 lines)** | Single round-trip |
| **Tool Calls Needed** | 2 calls | **1 call** | 50% fewer round-trips |

### 2. Distributed Edits (5 sites in a 1,000-line file)

Simulated editing 5 distinct locations across a 1,000-line file where each edit inserts 5 new lines, shifting subsequent line numbers by +5, +10, +15, and +20 lines.

| Metric | Traditional Line-Indexed Editing | `fast-file-editor` (`str_replace`) | Improvement |
| :--- | :--- | :--- | :--- |
| **Total Tool Latency** | ~15 - 20s (including LLM re-read turns) | **2.36 ms (all 5 edits)** | Instantaneous |
| **Re-reads Required** | **5 defensive re-reads** | **0 re-reads** | 100% eliminated |
| **Total Tool Calls** | **10 calls** (5 reads + 5 edits) | **5 calls** (5 consecutive edits) | 50% fewer turns |
| **Wasted Token Overhead** | **~50,000 tokens** (800 lines x 5 re-reads) | **0 tokens** | Zero context waste |
| **Line-drift Failure Rate** | High (fails if drift is not recalculated) | **0% (uniqueness-validated)** | Deterministic |

## Installation & Build

### Prerequisites

- Rust 1.80+ (`cargo`, `rustc`)

### Build Single Binary

```bash
git clone https://github.com/youngyunjeong/mcps.git
cd mcps/servers/filesystem/fast-file-editor
cargo build --release
```

The optimized, stripped binary (~390 KB) is produced at:
`target/release/fast-file-editor`

Copy it to your PATH:
```bash
cp target/release/fast-file-editor ~/.local/bin/
```

### Run Tests

```bash
cargo test
```

## Agent Configurations

### Antigravity (AGY)

Add to `~/.gemini/config/mcp_config.json`:

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "/Users/YOUR_USER/.local/bin/fast-file-editor",
      "args": []
    }
  }
}
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "fast-file-editor": {
      "command": "/Users/YOUR_USER/.local/bin/fast-file-editor",
      "args": []
    }
  }
}
```

### Cursor

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

## License

MIT
