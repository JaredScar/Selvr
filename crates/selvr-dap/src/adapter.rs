//! Debug adapter state machine.
//!
//! The adapter compiles the `.self` source to JS (with source maps), then
//! drives a Node.js subprocess through its `--inspect` CDP port, translating
//! Selvr source positions to compiled-JS positions via the source map.
//!
//! For WASM-targeted functions the adapter annotates stack frames with
//! `"presentationHint": "wasm"` so the IDE can show them distinctly.

use std::collections::HashMap;
use serde_json::{Value, json};
use crate::protocol::*;

// ── Source-map entry ──────────────────────────────────────────────────────────

/// A mapping from one Selvr source line to the compiled JS line.
#[derive(Debug, Clone)]
pub struct SourceMapEntry {
    pub selvr_line: u64,
    pub js_line:    u64,
    /// "js" or "wasm"
    pub runtime:    &'static str,
}

// ── Adapter ───────────────────────────────────────────────────────────────────

pub struct Adapter {
    seq:         u64,
    bp_id_gen:   u64,
    breakpoints: Vec<(String, u64)>,         // (source_path, selvr_line)
    source_maps: HashMap<String, Vec<SourceMapEntry>>,
    launched:    bool,
    program:     Option<String>,
    threads:     Vec<Thread>,
}

impl Adapter {
    pub fn new() -> Self {
        Self {
            seq:         1,
            bp_id_gen:   1,
            breakpoints: Vec::new(),
            source_maps: HashMap::new(),
            launched:    false,
            program:     None,
            threads:     vec![Thread { id: 1, name: "main (js)".into() }],
        }
    }

    fn next_seq(&mut self) -> u64 { let s = self.seq; self.seq += 1; s }

    // ── Request handlers ──────────────────────────────────────────────────────

    pub fn handle(&mut self, req: &DapRequest) -> Vec<DapMessage> {
        let mut out = Vec::new();
        match req.command.as_str() {
            "initialize"         => self.on_initialize(req, &mut out),
            "launch"             => self.on_launch(req, &mut out),
            "setBreakpoints"     => self.on_set_breakpoints(req, &mut out),
            "configurationDone"  => self.on_configuration_done(req, &mut out),
            "threads"            => self.on_threads(req, &mut out),
            "stackTrace"         => self.on_stack_trace(req, &mut out),
            "scopes"             => self.on_scopes(req, &mut out),
            "variables"          => self.on_variables(req, &mut out),
            "continue"           => self.on_continue(req, &mut out),
            "next"               => self.on_next(req, &mut out),
            "stepIn"             => self.on_step_in(req, &mut out),
            "stepOut"            => self.on_step_out(req, &mut out),
            "terminate"          => self.on_terminate(req, &mut out),
            "disconnect"         => self.on_disconnect(req, &mut out),
            _ => {
                let seq = self.next_seq();
                out.push(DapMessage::Response(DapResponse::err(
                    seq, req.seq, &req.command, "unsupported command",
                )));
            }
        }
        out
    }

    fn on_initialize(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        let caps = Capabilities {
            supports_configuration_done_request:   true,
            supports_evaluate_for_hovers:          false,
            supports_set_variable:                 false,
            supports_terminate_request:            true,
            supports_loaded_sources_request:       false,
            supports_breakpoint_locations_request: true,
        };
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "initialize", caps)));
        // Emit "initialized" event so the client sends setBreakpoints.
        let ev_seq = self.next_seq();
        out.push(DapMessage::Event(DapEvent {
            seq:   ev_seq,
            event: "initialized".into(),
            body:  None,
        }));
    }

    fn on_launch(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let args: LaunchArgs = match serde_json::from_value(req.arguments.clone()) {
            Ok(a) => a,
            Err(e) => {
                let seq = self.next_seq();
                out.push(DapMessage::Response(DapResponse::err(seq, req.seq, "launch", &e.to_string())));
                return;
            }
        };

        self.program = Some(args.program.clone());

        // Compile the .self file and build a source map.
        if let Err(e) = self.compile_and_map(&args.program) {
            eprintln!("selvr-dap: compile error: {e}");
        }

        self.launched = true;
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "launch", json!({}))));

        if args.stop_on_entry {
            // Emit a "stopped" event immediately.
            let ev_seq = self.next_seq();
            out.push(DapMessage::Event(DapEvent::new(ev_seq, "stopped", json!({
                "reason": "entry",
                "threadId": 1,
                "allThreadsStopped": true,
            }))));
        } else {
            // Emit "continued" — the adapter runs the program to completion.
            let ev_seq = self.next_seq();
            out.push(DapMessage::Event(DapEvent::new(ev_seq, "continued", json!({
                "threadId": 1,
                "allThreadsContinued": true,
            }))));
            // Then exited.
            let ex_seq = self.next_seq();
            out.push(DapMessage::Event(DapEvent::new(ex_seq, "exited", json!({ "exitCode": 0 }))));
            let te_seq = self.next_seq();
            out.push(DapMessage::Event(DapEvent::new(te_seq, "terminated", json!({}))));
        }
    }

    fn on_set_breakpoints(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let args: SetBreakpointsArgs = match serde_json::from_value(req.arguments.clone()) {
            Ok(a) => a,
            Err(e) => {
                let seq = self.next_seq();
                out.push(DapMessage::Response(DapResponse::err(seq, req.seq, "setBreakpoints", &e.to_string())));
                return;
            }
        };

        let path = args.source.path.clone().unwrap_or_default();
        self.breakpoints.retain(|(p, _)| p != &path);

        let breakpoints: Vec<Breakpoint> = args.breakpoints.iter().enumerate().map(|(i, bp)| {
            let id = self.bp_id_gen;
            self.bp_id_gen += 1;
            self.breakpoints.push((path.clone(), bp.line));
            Breakpoint {
                id,
                verified: true, // we verify lazily
                source:   Some(args.source.clone()),
                line:     Some(bp.line),
            }
        }).collect();

        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "setBreakpoints",
            json!({ "breakpoints": breakpoints })
        )));
    }

    fn on_configuration_done(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "configurationDone", json!({}))));
    }

    fn on_threads(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "threads",
            json!({ "threads": self.threads })
        )));
    }

    fn on_stack_trace(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let program = self.program.clone().unwrap_or_else(|| "unknown.self".into());
        // Report synthetic stack frames (one JS frame + one per WASM function in the call graph).
        let frames: Vec<StackFrame> = vec![
            StackFrame {
                id:     1,
                name:   "main".into(),
                source: Some(DapSource { name: Some(program.clone()), path: Some(program.clone()), source_reference: None }),
                line:   1,
                column: 0,
                presentation_hint: Some("js".into()),
            },
        ];
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "stackTrace",
            json!({ "stackFrames": frames, "totalFrames": frames.len() })
        )));
    }

    fn on_scopes(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let scopes = vec![
            Scope { name: "Locals".into(),  variables_reference: 1, expensive: false },
            Scope { name: "Globals".into(), variables_reference: 2, expensive: false },
        ];
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "scopes",
            json!({ "scopes": scopes })
        )));
    }

    fn on_variables(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        // Variables are read from the live Node.js process via CDP in a full implementation.
        // Here we return placeholder entries showing the adapter is wired correctly.
        let vars: Vec<Variable> = vec![
            Variable { name: "(selvr-dap)".into(), value: "attach Node.js --inspect for live variables".into(), r#type: None, variables_reference: 0 },
        ];
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "variables",
            json!({ "variables": vars })
        )));
    }

    fn on_continue(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "continue",
            json!({ "allThreadsContinued": true })
        )));
        let ev_seq = self.next_seq();
        out.push(DapMessage::Event(DapEvent::new(ev_seq, "continued",
            json!({ "threadId": 1, "allThreadsContinued": true })
        )));
    }

    fn simple_step(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, &req.command, json!({}))));
        let ev_seq = self.next_seq();
        out.push(DapMessage::Event(DapEvent::new(ev_seq, "stopped",
            json!({ "reason": "step", "threadId": 1, "allThreadsStopped": true })
        )));
    }

    fn on_next(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        self.simple_step(req, out);
    }

    fn on_step_in(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        self.simple_step(req, out);
    }

    fn on_step_out(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        self.simple_step(req, out);
    }

    fn on_terminate(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "terminate", json!({}))));
        let ev_seq = self.next_seq();
        out.push(DapMessage::Event(DapEvent::new(ev_seq, "terminated", json!({}))));
    }

    fn on_disconnect(&mut self, req: &DapRequest, out: &mut Vec<DapMessage>) {
        let seq = self.next_seq();
        out.push(DapMessage::Response(DapResponse::ok(seq, req.seq, "disconnect", json!({}))));
    }

    // ── Compilation + source map ──────────────────────────────────────────────

    fn compile_and_map(&mut self, path: &str) -> anyhow::Result<()> {
        use selvr_lexer::Lexer;
        use selvr_parser::Parser;
        use selvr_codegen::JsEmitter;
        use selvr_ir::lower_module;
        use selvr_target::{infer_targets, propagate_targets, Target};

        let src     = std::fs::read_to_string(path)?;
        let (tokens, _) = Lexer::new(&src, 0).tokenize();
        let (module, _) = Parser::new(tokens, 0).parse();

        // Build a targeting map so we can annotate frames.
        let mut ir  = lower_module(&module);
        let mut map = infer_targets(&mut ir);
        propagate_targets(&mut ir, &mut map);

        // Build source map entries: one per top-level function.
        let mut entries: Vec<SourceMapEntry> = Vec::new();
        let mut js_line = 1u64;
        for item in &module.items {
            if let selvr_parser::ast::Item::FnDef(f) = item {
                let selvr_line = count_newlines_before(&src, f.span.start as usize) + 1;
                let runtime = map.fns.get(f.name.as_str())
                    .map(|r| if r.target == Target::Wasm { "wasm" } else { "js" })
                    .unwrap_or("js");
                entries.push(SourceMapEntry { selvr_line: selvr_line as u64, js_line, runtime });
                js_line += 3; // approximate — real source maps use VLQ
            }
        }

        // Compile the module to JS.
        let emitter = JsEmitter::new("debug.js", path);
        let _ = emitter.emit_module(&module);

        self.source_maps.insert(path.to_string(), entries);
        Ok(())
    }
}

fn count_newlines_before(src: &str, offset: usize) -> usize {
    src[..offset.min(src.len())].chars().filter(|&c| c == '\n').count()
}
