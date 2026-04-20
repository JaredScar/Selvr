//! DAP JSON-RPC message types.
//!
//! Reference: <https://microsoft.github.io/debug-adapter-protocol/specification>

use serde::{Serialize, Deserialize};
use serde_json::Value;

// ── Wire framing ──────────────────────────────────────────────────────────────

/// A DAP message envelope.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DapMessage {
    Request(DapRequest),
    Response(DapResponse),
    Event(DapEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DapRequest {
    pub seq:       u64,
    pub command:   String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DapResponse {
    pub seq:         u64,
    pub request_seq: u64,
    pub success:     bool,
    pub command:     String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message:     Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body:        Option<Value>,
}

impl DapResponse {
    pub fn ok(seq: u64, request_seq: u64, command: &str, body: impl Serialize) -> Self {
        Self {
            seq,
            request_seq,
            success: true,
            command: command.into(),
            message: None,
            body:    Some(serde_json::to_value(body).unwrap_or(Value::Null)),
        }
    }

    pub fn err(seq: u64, request_seq: u64, command: &str, msg: &str) -> Self {
        Self {
            seq,
            request_seq,
            success: false,
            command: command.into(),
            message: Some(msg.into()),
            body:    None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DapEvent {
    pub seq:   u64,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body:  Option<Value>,
}

impl DapEvent {
    pub fn new(seq: u64, event: &str, body: impl Serialize) -> Self {
        Self {
            seq,
            event: event.into(),
            body:  Some(serde_json::to_value(body).unwrap_or(Value::Null)),
        }
    }
}

// ── Common body types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub supports_configuration_done_request: bool,
    pub supports_evaluate_for_hovers:        bool,
    pub supports_set_variable:               bool,
    pub supports_terminate_request:          bool,
    pub supports_loaded_sources_request:     bool,
    pub supports_breakpoint_locations_request: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeArgs {
    pub adapter_id:                   String,
    #[serde(default)]
    pub lines_start_at1:              bool,
    #[serde(default)]
    pub columns_start_at1:            bool,
    pub path_format:                  Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchArgs {
    pub program:    String,
    #[serde(default)]
    pub stop_on_entry: bool,
    /// JS or hybrid
    #[serde(default = "default_runtime")]
    pub runtime:    String,
}

fn default_runtime() -> String { "js".into() }

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    pub line:      u64,
    pub column:    Option<u64>,
    pub condition: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsArgs {
    pub source:      DapSource,
    pub breakpoints: Vec<SourceBreakpoint>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DapSource {
    pub name:   Option<String>,
    pub path:   Option<String>,
    pub source_reference: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    pub id:       u64,
    pub verified: bool,
    pub source:   Option<DapSource>,
    pub line:     Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub id:     u64,
    pub name:   String,
    pub source: Option<DapSource>,
    pub line:   u64,
    pub column: u64,
    /// "js" or "wasm"
    pub presentation_hint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scope {
    pub name:                 String,
    pub variables_reference:  u64,
    pub expensive:            bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variable {
    pub name:                String,
    pub value:               String,
    pub r#type:              Option<String>,
    pub variables_reference: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thread {
    pub id:   u64,
    pub name: String,
}
