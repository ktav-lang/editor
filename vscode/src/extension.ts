import * as fs from "fs";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import { resolveServerPath as resolveServerPathPure } from "./discovery";
import { registerKeyDecorations } from "./key-decorations";
import { reattachRestoredEditors } from "./restored-editors";

let shutdownClient: (() => Promise<void>) | undefined;

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

function buildClient(command: string, outputChannel: vscode.OutputChannel): LanguageClient {
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
  reattachRestoredEditors(
    { window: vscode.window, workspace: vscode.workspace, commands: vscode.commands, TabInputText: vscode.TabInputText },
    process.uptime(),
  );
  const outputChannel = vscode.window.createOutputChannel(
    "Ktav Language Server",
  );
  context.subscriptions.push(outputChannel);

  let client: LanguageClient | undefined;
  let closing = false;
  let lifecycle = Promise.resolve();
  let shutdownPromise: Promise<void> | undefined;
  let configurationRevision = 0;

  // Serialize startup, replacement and shutdown, including failed operations.
  function enqueue(operation: () => Promise<void>): Promise<void> {
    const next = lifecycle.then(operation);
    lifecycle = next.catch(() => {});
    return next;
  }

  async function disposeClient(): Promise<void> {
    const previous = client;
    client = undefined;
    if (!previous) {
      return;
    }
    try {
      await previous.dispose(2000);
    } catch (err) {
      if (previous.needsStop()) {
        client = previous;
        throw err;
      }
      // v9 rejects stop after a failed spawn, leaving its diagnostics allocated.
      previous.diagnostics?.dispose();
      outputChannel.appendLine(`[ktav] disposal failed: ${(err as Error)?.message ?? err}`);
    }
  }

  shutdownClient = () => {
    closing = true;
    return shutdownPromise ??= enqueue(disposeClient);
  };

  function restartServer(revision?: number): Promise<void> {
    return enqueue(async () => {
      if (closing || (revision !== undefined && revision !== configurationRevision)) {
        return;
      }
      outputChannel.appendLine("[ktav] restarting language server");
      try {
        await disposeClient();
        if (closing) {
          return;
        }
        // A client's serverOptions are immutable; rediscover on every restart.
        client = buildClient(resolveServerPath(context), outputChannel);
        await client.start();
        outputChannel.appendLine("[ktav] language server started");
        refreshDecorations();
      } catch (err) {
        outputChannel.appendLine(`[ktav] restart failed: ${(err as Error)?.message ?? err}`);
        if (!closing) {
          void vscode.window.showErrorMessage(
            `Ktav: restart failed: ${(err as Error)?.message ?? err}`,
          );
        }
      }
    });
  }

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

  const refreshDecorations = registerKeyDecorations(context, () => client);

  // Restart command — surfaces in the Command Palette as
  // "Ktav: Restart Language Server".
  context.subscriptions.push(
    vscode.commands.registerCommand("ktav.restartServer", () => restartServer()),
  );

  // If `ktav.server.path` changes, the running server still points at the
  // old binary — prompt the user to restart.
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (e) => {
      if (closing || !e.affectsConfiguration("ktav.server.path")) {
        return;
      }
      const revision = ++configurationRevision;
      const choice = await vscode.window.showInformationMessage(
        "Ktav: server path changed. Restart the language server now?",
        "Restart",
      );
      if (choice === "Restart") {
        await restartServer(revision);
      }
    }),
  );

  await enqueue(async () => {
    if (closing) {
      return;
    }
    let command = "ktav-lsp";
    try {
      command = resolveServerPath(context);
      client = buildClient(command, outputChannel);
      await client.start();
      outputChannel.appendLine("[ktav] language server started");
      refreshDecorations();
    } catch (err) {
      const msg =
        `Ktav: failed to start language server "${command}". ` +
        `Install it via \`cargo install ktav-lsp\` or set \`ktav.server.path\` ` +
        `in your settings to point at an existing binary.`;
      outputChannel.appendLine(`[ktav] ${msg}`);
      outputChannel.appendLine(`[ktav] cause: ${(err as Error)?.message ?? err}`);
      if (!closing) {
        void vscode.window.showErrorMessage(msg);
      }
    }
  });
}

export function deactivate(): Thenable<void> | undefined {
  // Bounded shutdown: don't let a stuck server block VS Code reload.
  return shutdownClient?.();
}
