use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

#[derive(Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Serialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[fast-file-editor] JSON parse error: {}", e);
                continue;
            }
        };

        let id = match req.id {
            Some(i) => i,
            None => {
                // Notification, no response required
                continue;
            }
        };

        let resp = handle_request(&req.method, req.params, id);
        let resp_json = serde_json::to_string(&resp)?;
        writeln!(stdout, "{}", resp_json)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_request(method: &str, params: Option<Value>, id: Value) -> JsonRpcResponse {
    match method {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "fast-file-editor",
                    "version": "0.1.0"
                }
            })),
            error: None,
        },
        "ping" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "str_replace",
                        "description": "Fast atomic string replacement tool (Claude Code style). Replaces a unique target string with replacement text in a file. The target old_str must occur exactly once in the file to prevent unintended modifications.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Absolute path to the target file to modify."
                                },
                                "old_str": {
                                    "type": "string",
                                    "description": "The exact unique substring to replace. Must occur exactly once in the file."
                                },
                                "new_str": {
                                    "type": "string",
                                    "description": "The replacement substring."
                                }
                            },
                            "required": ["path", "old_str", "new_str"]
                        }
                    },
                    {
                        "name": "view_file_fast",
                        "description": "Fast file viewer with 1-indexed line numbers. Does not suffer from 800-line or 45KB payload truncation limits.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Absolute path to the file to view."
                                },
                                "start_line": {
                                    "type": "integer",
                                    "description": "Optional 1-indexed starting line number (default: 1)."
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Optional maximum number of lines to return (default: 2000)."
                                }
                            },
                            "required": ["path"]
                        }
                    }
                ]
            })),
            error: None,
        },
        "tools/call" => {
            let params = params.unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

            let (result_text, is_error) = match tool_name {
                "str_replace" => execute_str_replace(&arguments),
                "view_file_fast" => execute_view_file(&arguments),
                unknown => (format!("Unknown tool: {}", unknown), true),
            };

            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": result_text
                        }
                    ],
                    "isError": is_error
                })),
                error: None,
            }
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": format!("Method '{}' not found", method)
            })),
        },
    }
}

fn execute_str_replace(args: &Value) -> (String, bool) {
    let path_str = match args.get("path").and_then(Value::as_str) {
        Some(p) => p,
        None => return ("Missing required argument: 'path'".to_string(), true),
    };

    let old_str = match args.get("old_str").and_then(Value::as_str) {
        Some(s) => s,
        None => return ("Missing required argument: 'old_str'".to_string(), true),
    };

    let new_str = match args.get("new_str").and_then(Value::as_str) {
        Some(s) => s,
        None => return ("Missing required argument: 'new_str'".to_string(), true),
    };

    if old_str.is_empty() {
        return ("Argument 'old_str' cannot be empty.".to_string(), true);
    }

    let path = Path::new(path_str);
    if !path.exists() {
        return (format!("File '{}' does not exist.", path_str), true);
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return (format!("Failed to read file '{}': {}", path_str, e), true),
    };

    let matches: Vec<usize> = content.match_indices(old_str).map(|(i, _)| i).collect();
    let count = matches.len();

    if count == 0 {
        return (
            format!(
                "Error: 'old_str' was not found in '{}'. Please check exact characters, indentation, and newlines.",
                path_str
            ),
            true,
        );
    }

    if count > 1 {
        return (
            format!(
                "Error: 'old_str' appears {} times in '{}'. Please provide more surrounding context lines to make it uniquely identifiable.",
                count, path_str
            ),
            true,
        );
    }

    let pos = matches[0];
    let line_num = content[..pos].bytes().filter(|&b| b == b"
"[0]).count() + 1;

    let mut new_content = String::with_capacity(content.len() + new_str.len().saturating_sub(old_str.len()));
    new_content.push_str(&content[..pos]);
    new_content.push_str(new_str);
    new_content.push_str(&content[pos + old_str.len()..]);

    // Atomic write
    let temp_path = format!("{}.tmp.{}", path_str, std::process::id());
    if let Err(e) = fs::write(&temp_path, &new_content) {
        return (format!("Failed to write temporary file: {}", e), true);
    }

    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return (format!("Failed to atomically replace file '{}': {}", path_str, e), true);
    }

    (
        format!(
            "Successfully replaced 1 occurrence at line {} in '{}'.",
            line_num, path_str
        ),
        false,
    )
}

fn execute_view_file(args: &Value) -> (String, bool) {
    let path_str = match args.get("path").and_then(Value::as_str) {
        Some(p) => p,
        None => return ("Missing required argument: 'path'".to_string(), true),
    };

    let start_line = args.get("start_line").and_then(Value::as_u64).unwrap_or(1).max(1) as usize;
    let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(2000).max(1) as usize;

    let path = Path::new(path_str);
    if !path.exists() {
        return (format!("File '{}' does not exist.", path_str), true);
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return (format!("Failed to read file '{}': {}", path_str, e), true),
    };

    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    if total_lines == 0 {
        return (format!("File '{}' is empty (0 lines).", path_str), false);
    }

    let start_idx = (start_line - 1).min(total_lines);
    let end_idx = (start_idx + limit).min(total_lines);

    let mut output = String::new();
    output.push_str(&format!(
        "File: {} (total lines: {})
Showing lines {} to {}:

",
        path_str, total_lines, start_idx + 1, end_idx
    ));

    for (i, line) in lines[start_idx..end_idx].iter().enumerate() {
        let current_line_num = start_idx + i + 1;
        output.push_str(&format!("{:>5}: {}
", current_line_num, line));
    }

    (output, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_temp_file(content: &str) -> (tempfile::NamedTempFile, String) {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        let path = file.path().to_str().unwrap().to_string();
        (file, path)
    }

    #[test]
    fn test_str_replace_success() {
        let (_file, path) = create_temp_file("fn alpha() {}\nfn beta() {}\nfn gamma() {}\n");
        let args = json!({
            "path": path,
            "old_str": "fn beta() {}",
            "new_str": "fn beta_modified() {}"
        });

        let (msg, is_error) = execute_str_replace(&args);
        assert!(!is_error);
        assert!(msg.contains("Successfully replaced 1 occurrence at line 2"));

        let updated = fs::read_to_string(&path).unwrap();
        assert_eq!(updated, "fn alpha() {}\nfn beta_modified() {}\nfn gamma() {}\n");
    }

    #[test]
    fn test_str_replace_not_found() {
        let (_file, path) = create_temp_file("const a = 1;\nconst b = 2;\n");
        let args = json!({
            "path": path,
            "old_str": "const c = 3;",
            "new_str": "const c = 4;"
        });

        let (msg, is_error) = execute_str_replace(&args);
        assert!(is_error);
        assert!(msg.contains("was not found"));
    }

    #[test]
    fn test_str_replace_duplicate() {
        let (_file, path) = create_temp_file("item\nother\nitem\n");
        let args = json!({
            "path": path,
            "old_str": "item",
            "new_str": "item_new"
        });

        let (msg, is_error) = execute_str_replace(&args);
        assert!(is_error);
        assert!(msg.contains("appears 2 times"));
    }

    #[test]
    fn test_view_file_fast() {
        let (_file, path) = create_temp_file("line 1\nline 2\nline 3\nline 4\nline 5\n");
        let args = json!({
            "path": path,
            "start_line": 2,
            "limit": 3
        });

        let (output, is_error) = execute_view_file(&args);
        assert!(!is_error);
        assert!(output.contains("Showing lines 2 to 4"));
        assert!(output.contains("    2: line 2"));
        assert!(output.contains("    4: line 4"));
        assert!(!output.contains("line 1"));
        assert!(!output.contains("line 5"));
    }
}
