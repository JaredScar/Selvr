/**
 * Selvr VS Code Extension — Language Client
 *
 * Responsibilities:
 *  1. Start `selvr-lsp` as a child process (stdin/stdout transport).
 *  2. Register target-annotation decorations (⚙ WASM / ⚡ JS).
 *  3. Register commands: restartServer, showTargetMap, formatDocument.
 *  4. Wire the DAP adapter for debugging .self files.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
  Executable,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

// ── Decoration types ──────────────────────────────────────────────────────────

const wasmDecoration = vscode.window.createTextEditorDecorationType({
  after: {
    contentText: ' ⚙ wasm',
    color:       new vscode.ThemeColor('charts.purple'),
    fontStyle:   'italic',
    margin:      '0 0 0 8px',
  },
});

const jsDecoration = vscode.window.createTextEditorDecorationType({
  after: {
    contentText: ' ⚡ js',
    color:       new vscode.ThemeColor('charts.yellow'),
    fontStyle:   'italic',
    margin:      '0 0 0 8px',
  },
});

// ── Activation ────────────────────────────────────────────────────────────────

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await startLanguageServer(context);
  registerCommands(context);
  registerDecorationRefresh(context);
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}

// ── Language server ───────────────────────────────────────────────────────────

async function startLanguageServer(context: vscode.ExtensionContext): Promise<void> {
  const config    = vscode.workspace.getConfiguration('selvr');
  const serverBin = config.get<string>('serverPath') ?? 'selvr-lsp';

  const serverExecutable: Executable = {
    command:   serverBin,
    transport: TransportKind.stdio,
    options:   { env: { ...process.env } },
  };

  const serverOptions: ServerOptions = {
    run:   serverExecutable,
    debug: serverExecutable,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'selvr' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.self'),
    },
    outputChannelName: 'Selvr Language Server',
    traceOutputChannel: vscode.window.createOutputChannel('Selvr LSP Trace'),
  };

  client = new LanguageClient(
    'selvr',
    'Selvr Language Server',
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);
  await client.start();
  vscode.window.setStatusBarMessage('$(check) Selvr LSP ready', 3000);
}

// ── Commands ──────────────────────────────────────────────────────────────────

function registerCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand('selvr.restartServer', async () => {
      if (client) { await client.stop(); }
      await startLanguageServer(context);
    }),

    vscode.commands.registerCommand('selvr.showTargetMap', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== 'selvr') {
        vscode.window.showWarningMessage('Open a .self file to see the target map.');
        return;
      }
      // Invoke `selvr explain <file> --json` as a child process.
      const { execFile } = await import('child_process');
      execFile('selvr', ['explain', editor.document.fileName, '--json'], (err, stdout) => {
        if (err) {
          vscode.window.showErrorMessage(`selvr explain failed: ${err.message}`);
          return;
        }
        const panel = vscode.window.createWebviewPanel(
          'selvrTargetMap',
          'Selvr Target Map',
          vscode.ViewColumn.Beside,
          {},
        );
        panel.webview.html = buildTargetMapHtml(stdout);
      });
    }),

    vscode.commands.registerCommand('selvr.formatDocument', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== 'selvr') { return; }
      await vscode.commands.executeCommand('editor.action.formatDocument');
    }),
  );
}

// ── Target-annotation decorations ─────────────────────────────────────────────

function registerDecorationRefresh(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration('selvr');
  if (!config.get<boolean>('showTargetAnnotations', true)) { return; }

  const refresh = (editor: vscode.TextEditor | undefined) => {
    if (!editor || editor.document.languageId !== 'selvr') { return; }
    applyTargetDecorations(editor);
  };

  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor(refresh),
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (vscode.window.activeTextEditor?.document === doc) {
        refresh(vscode.window.activeTextEditor);
      }
    }),
  );

  refresh(vscode.window.activeTextEditor);
}

function applyTargetDecorations(editor: vscode.TextEditor): void {
  // Parse function declarations from the document text with a simple regex.
  // The LSP hover already provides full targeting data; decorations are
  // a quick best-effort based on the heuristic attribute annotations.
  const text  = editor.document.getText();
  const lines = text.split('\n');

  const wasmRanges: vscode.DecorationOptions[] = [];
  const jsRanges:   vscode.DecorationOptions[] = [];

  // Track the attribute of the next fn.
  let nextTarget: 'wasm' | 'js' | 'auto' = 'auto';

  lines.forEach((line, i) => {
    const attrMatch = line.match(/^\s*#\[(wasm|js)\]/);
    if (attrMatch) {
      nextTarget = attrMatch[1] as 'wasm' | 'js';
      return;
    }
    const fnMatch = line.match(/^\s*(?:export\s+)?(?:async\s+)?fn\s+(\w+)/);
    if (fnMatch) {
      const range = new vscode.Range(i, line.indexOf('fn'), i, line.length);
      if (nextTarget === 'wasm') {
        wasmRanges.push({ range });
      } else if (nextTarget === 'js') {
        jsRanges.push({ range });
      } else {
        // Heuristic: if it mentions "document.", "window.", "addEventListener" → js
        const body = lines.slice(i, i + 20).join('\n');
        if (/\b(document|window|addEventListener|innerHTML|DOM)\b/.test(body)) {
          jsRanges.push({ range });
        } else if (/\b(Math\.(sin|cos|sqrt|pow|abs|floor)|for|while)/.test(body)) {
          wasmRanges.push({ range });
        }
      }
      nextTarget = 'auto';
    }
  });

  editor.setDecorations(wasmDecoration, wasmRanges);
  editor.setDecorations(jsDecoration,   jsRanges);
}

// ── Target map webview ────────────────────────────────────────────────────────

function buildTargetMapHtml(json: string): string {
  let rows = '';
  try {
    const data = JSON.parse(json) as Record<string, { target: string; score: number; reason: string; forced: boolean }>;
    for (const [name, rec] of Object.entries(data)) {
      const icon  = rec.target === 'wasm' ? '⚙' : '⚡';
      const color = rec.target === 'wasm' ? '#a5b4fc' : '#fbbf24';
      const badge = rec.forced ? ' <span style="font-size:10px;opacity:.7">(forced)</span>' : '';
      rows += `<tr>
        <td style="color:${color};font-size:18px">${icon}</td>
        <td><code>${name}</code>${badge}</td>
        <td style="color:${color}">${rec.target.toUpperCase()}</td>
        <td style="color:#94a3b8">${rec.score}</td>
        <td style="color:#94a3b8;font-size:12px">${rec.reason}</td>
      </tr>`;
    }
  } catch {
    rows = `<tr><td colspan="5">No targeting data available (run <code>selvr explain</code> first)</td></tr>`;
  }

  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<style>
  body { font-family: 'Segoe UI', sans-serif; background:#0d0f14; color:#e2e8f0; padding:20px; }
  h1 { color:#00d4aa; font-size:18px; margin-bottom:16px; }
  table { border-collapse:collapse; width:100%; }
  th { text-align:left; padding:8px 12px; border-bottom:1px solid #232736; color:#64748b; font-size:11px; text-transform:uppercase; letter-spacing:.8px; }
  td { padding:8px 12px; border-bottom:1px solid #1a1d27; vertical-align:top; }
  tr:hover td { background:#13161f; }
  code { color:#fcd34d; }
</style>
</head>
<body>
<h1>⚙ Selvr WASM / JS Target Map</h1>
<table>
  <thead><tr><th></th><th>Function</th><th>Target</th><th>Score</th><th>Reason</th></tr></thead>
  <tbody>${rows}</tbody>
</table>
</body>
</html>`;
}
