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
