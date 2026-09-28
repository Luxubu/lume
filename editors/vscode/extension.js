// Starts `lume lsp` for .lume files: Lume's own errors and warnings appear
// in the editor as you type. Highlighting needs nothing from here; it comes
// from syntaxes/lume.tmLanguage.json.
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");

let client;

function activate(context) {
  const command = vscode.workspace.getConfiguration("lume").get("path") || "lume";
  const folder = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const server = { command, args: ["lsp"], options: folder ? { cwd: folder } : {} };
  client = new LanguageClient("lume", "Lume", server, {
    documentSelector: [{ scheme: "file", language: "lume" }],
  });
  client.start().catch((e) => {
    vscode.window.showWarningMessage(
      `Lume: could not start \`${command} lsp\` (${e.message}). Set "lume.path" to the lume command.`
    );
  });
  context.subscriptions.push({ dispose: () => client && client.stop() });
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
