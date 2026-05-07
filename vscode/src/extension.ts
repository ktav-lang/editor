import * as fs from "fs";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerPath as resolveServerPathPure } from "./discovery";

let client: LanguageClient | undefined;

function resolveServerPath(context: vscode.ExtensionContext): string {
  return resolveServerPathPure({
    getServerPathSetting: () =>
      vscode.workspace.getConfiguration("ktav").get<string>("server.path") ?? "",
    existsSync: fs.existsSync,
    platform: process.platform,
    arch: process.arch,
    extensionPath: context.extensionPath,
  });
}

function buildClient(context: vscode.ExtensionContext, outputChannel: vscode.OutputChannel): LanguageClient {
  const command = resolveServerPath(context);
  outputChannel.appendLine(`[ktav] resolved server command: ${command}`);

  const serverOptions: ServerOptions = {
    run: { command, transport: TransportKind.stdio },
    debug: { command, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ktav" }],
    outputChannel,
    // vscode-languageclient v9 reads `${id}.trace.server` automatically when
    // a traceOutputChannel is supplied — wiring this enables the
    // `ktav.trace.server` setting declared in package.json.
    traceOutputChannel: outputChannel,
    synchronize: {
      configurationSection: "ktav",
    },
  };

  return new LanguageClient(
    "ktav",
    "Ktav Language Server",
    serverOptions,
    clientOptions,
  );
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const outputChannel = vscode.window.createOutputChannel(
    "Ktav Language Server",
  );
  context.subscriptions.push(outputChannel);

  client = buildClient(context, outputChannel);

  // Register our extension as the formatter for `.ktav` explicitly.
  // LanguageClient also creates a formatter from LSP capabilities, but VS
  // Code's `editor.defaultFormatter` lookup needs an extension-owned
  // provider with our publisher.name ID; the LSP-derived one isn't
  // attributed to us. Without this, users see "no formatter for ktav".
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider(
      { language: "ktav", scheme: "file" },
      {
        async provideDocumentFormattingEdits(
          document: vscode.TextDocument,
          options: vscode.FormattingOptions,
          token: vscode.CancellationToken,
        ): Promise<vscode.TextEdit[]> {
          if (!client || !client.isRunning()) {
            outputChannel.appendLine("[ktav] format requested but client not running");
            return [];
          }
          try {
            const result: any[] | null = await client.sendRequest(
              "textDocument/formatting",
              {
                textDocument: { uri: document.uri.toString() },
                options: {
                  tabSize: options.tabSize,
                  insertSpaces: options.insertSpaces,
                },
              },
              token,
            );
            if (!result || result.length === 0) {
              return [];
            }
            return result.map(
              (e) =>
                new vscode.TextEdit(
                  new vscode.Range(
                    new vscode.Position(e.range.start.line, e.range.start.character),
                    new vscode.Position(e.range.end.line, e.range.end.character),
                  ),
                  e.newText,
                ),
            );
          } catch (err) {
            outputChannel.appendLine(
              `[ktav] format failed: ${(err as Error)?.message ?? err}`,
            );
            return [];
          }
        },
      },
    ),
  );

  try {
    await client.start();
    outputChannel.appendLine("[ktav] language server started");
  } catch (err) {
    const command = resolveServerPath(context);
    const msg =
      `Ktav: failed to start language server "${command}". ` +
      `Install it via \`cargo install ktav-lsp\` or set \`ktav.server.path\` ` +
      `in your settings to point at an existing binary.`;
    outputChannel.appendLine(`[ktav] ${msg}`);
    outputChannel.appendLine(`[ktav] cause: ${(err as Error)?.message ?? err}`);
    void vscode.window.showErrorMessage(msg);
    // Keep `client` around so the default ErrorHandler / restart command
    // can still resurrect it; do NOT null it out on first failure.
  }

  // Restart command — surfaces in the Command Palette as
  // "Ktav: Restart Language Server".
  context.subscriptions.push(
    vscode.commands.registerCommand("ktav.restartServer", async () => {
      if (!client) {
        outputChannel.appendLine("[ktav] restart requested but no client; rebuilding");
        client = buildClient(context, outputChannel);
        try {
          await client.start();
        } catch (err) {
          void vscode.window.showErrorMessage(
            `Ktav: restart failed: ${(err as Error)?.message ?? err}`,
          );
        }
        return;
      }
      outputChannel.appendLine("[ktav] restarting language server");
      try {
        await client.restart();
      } catch (err) {
        void vscode.window.showErrorMessage(
          `Ktav: restart failed: ${(err as Error)?.message ?? err}`,
        );
      }
    }),
  );

  // If `ktav.server.path` changes, the running server still points at the
  // old binary — prompt the user to restart.
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (!e.affectsConfiguration("ktav.server.path")) {
        return;
      }
      const choice = await vscode.window.showInformationMessage(
        "Ktav: server path changed. Restart the language server now?",
        "Restart",
      );
      if (choice === "Restart" && client) {
        try {
          await client.restart();
        } catch (err) {
          void vscode.window.showErrorMessage(
            `Ktav: restart failed: ${(err as Error)?.message ?? err}`,
          );
        }
      }
    }),
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  // Bounded shutdown: don't let a stuck server block VS Code reload.
  return client.stop(2000);
}
