import * as assert from "assert";
import * as fs from "fs";
import * as path from "path";
import { createRequire } from "module";
import { runInNewContext } from "vm";
import type * as vscode from "vscode";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

function harness() {
  const extensionPath = path.join(path.sep, "extension-fixture");
  const bundled = path.join(
    extensionPath, "bin", `${process.platform}-${process.arch}`,
    process.platform === "win32" ? "ktav-lsp.exe" : "ktav-lsp",
  );
  const clients: FakeClient[] = [];
  const created = new Map<number, ReturnType<typeof deferred<FakeClient>>>();
  const events: string[] = [];
  const errors: string[] = [];
  const existing = new Set(["server-A", "server-B", "server-C"]);
  const failCommands = new Set<string>();
  const startGates = new Map<string, ReturnType<typeof deferred<void>>>();
  const prompts: ReturnType<typeof deferred<string | undefined>>[] = [];
  const commands = new Map<string, () => Promise<void>>();
  const configurationHandlers = new Set<(e: vscode.ConfigurationChangeEvent) => Promise<void>>();
  const formatters = new Set<vscode.DocumentFormattingEditProvider>();
  let setting = "server-A";
  let activeResources = 0;
  let diagnosticResources = 0;
  let maxActiveResources = 0;
  let outputDisposals = 0;
  const output = {
    appendLine: (_line: string) => {},
    dispose: () => { outputDisposals++; },
  };

  class FakeClient {
    running = false;
    disposeCalls = 0;
    restartCalls = 0;
    stopTimeouts: number[] = [];
    stopGate: ReturnType<typeof deferred<void>> | undefined;
    stopFailure: Error | undefined;
    startFailed = false;
    diagnostics: { dispose: () => void } | undefined;
    readonly startEntered = deferred<void>();
    readonly stopEntered = deferred<void>();

    constructor(
      readonly id: string,
      _name: string,
      readonly serverOptions: {
        run: { command: string; transport: number };
        debug: { command: string; transport: number };
      },
      readonly clientOptions: {
        outputChannel: unknown;
        traceOutputChannel: unknown;
        synchronize: { configurationSection: string };
      },
    ) {
      clients.push(this);
      events.push(`create:${this.command}`);
      created.get(clients.length - 1)?.resolve(this);
    }

    get command() { return this.serverOptions.run.command; }
    isRunning() { return this.running; }
    needsStop() { return this.running; }

    async start() {
      events.push(`start:${this.command}`);
      this.startEntered.resolve();
      if (!this.diagnostics) {
        diagnosticResources++;
        let disposed = false;
        this.diagnostics = { dispose: () => {
          if (!disposed) { diagnosticResources--; disposed = true; }
        } };
      }
      await startGates.get(this.command)?.promise;
      if (failCommands.has(this.command)) {
        this.startFailed = true;
        throw new Error(`spawn failed: ${this.command}`);
      }
      this.running = true;
      activeResources++;
      maxActiveResources = Math.max(maxActiveResources, activeResources);
      events.push(`started:${this.command}`);
    }

    async stop(timeout = 2000) {
      this.stopTimeouts.push(timeout);
      events.push(`stop:${this.command}`);
      if (this.stopFailure) {
        const error = this.stopFailure;
        this.stopFailure = undefined;
        throw error;
      }
      if (this.startFailed) {
        throw new Error("Client is not running and can't be stopped. It's current state is: startFailed");
      }
      if (this.running) {
        this.running = false;
        activeResources--;
      }
      this.stopEntered.resolve();
      await this.stopGate?.promise;
      this.diagnostics?.dispose();
      this.diagnostics = undefined;
      events.push(`stopped:${this.command}`);
    }

    async dispose(timeout: number) {
      this.disposeCalls++;
      await this.stop(timeout);
    }

    async restart() {
      this.restartCalls++;
      await this.stop();
      await this.start();
    }

    async sendRequest() {
      events.push(`format:${this.command}`);
      return [];
    }
  }

  function register<T>(set: Set<T>, value: T) {
    set.add(value);
    return { dispose: () => { set.delete(value); } };
  }

  const mockVscode = {
    window: {
      createOutputChannel: () => output,
      showErrorMessage: async (message: string) => { errors.push(message); },
      showInformationMessage: () => {
        const prompt = deferred<string | undefined>();
        prompts.push(prompt);
        return prompt.promise;
      },
    },
    workspace: {
      getConfiguration: () => ({ get: () => setting }),
      onDidChangeConfiguration: (handler: (e: vscode.ConfigurationChangeEvent) => Promise<void>) =>
        register(configurationHandlers, handler),
    },
    languages: {
      registerDocumentFormattingEditProvider: (_selector: unknown, provider: vscode.DocumentFormattingEditProvider) =>
        register(formatters, provider),
    },
    commands: {
      registerCommand: (name: string, handler: () => Promise<void>) => {
        commands.set(name, handler);
        return { dispose: () => { commands.delete(name); } };
      },
    },
  };
  const context = { extensionPath, subscriptions: [] as vscode.Disposable[] };
  const filename = require.resolve("../../extension");
  const realRequire = createRequire(filename);
  const exports = {} as typeof import("../../extension");

  // Isolate module state and VS Code imports without patching Node's loader.
  runInNewContext(fs.readFileSync(filename, "utf8"), {
    exports,
    process,
    require: (name: string) => {
      if (name === "vscode") { return mockVscode; }
      if (name === "fs") { return { existsSync: (p: string) => existing.has(p) }; }
      if (name === "vscode-languageclient/node") {
        return { LanguageClient: FakeClient, TransportKind: { stdio: 0 } };
      }
      // Editor decorations are not part of the client lifecycle.
      if (name === "./key-decorations") { return { registerKeyDecorations: () => () => {} }; }
      if (name === "./restored-editors") { return { reattachRestoredEditors: () => {} }; }
      return realRequire(name);
    },
  }, { filename });

  return {
    clients, events, errors, existing, failCommands, startGates, prompts, commands,
    configurationHandlers, formatters, context, bundled, output,
    get activeResources() { return activeResources; },
    get diagnosticResources() { return diagnosticResources; },
    get maxActiveResources() { return maxActiveResources; },
    get outputDisposals() { return outputDisposals; },
    activate: () => exports.activate(context as unknown as vscode.ExtensionContext),
    clientCreated(index: number) {
      if (clients[index]) { return Promise.resolve(clients[index]); }
      const signal = deferred<FakeClient>();
      created.set(index, signal);
      return signal.promise;
    },
    deactivate: () => Promise.resolve(exports.deactivate()),
    restart: () => commands.get("ktav.restartServer")!(),
    change(value: string, section = "ktav.server.path") {
      setting = value;
      return Promise.all([...configurationHandlers].map((handler) => handler({
        affectsConfiguration: (name: string) => name === section,
      })));
    },
    async format() {
      const provider = [...formatters][0];
      await provider.provideDocumentFormattingEdits(
        { uri: { toString: () => "file:///fixture.ktav" } } as vscode.TextDocument,
        { tabSize: 4, insertSpaces: true },
        {} as vscode.CancellationToken,
      );
    },
    disposeSubscriptions() {
      context.subscriptions.forEach((subscription) => subscription.dispose());
    },
  };
}

suite("extension: language client lifecycle", () => {
  let h: ReturnType<typeof harness>;

  setup(() => { h = harness(); });
  teardown(async () => {
    await h.deactivate();
    h.disposeSubscriptions();
    assert.strictEqual(h.activeResources, 0);
    assert.strictEqual(h.diagnosticResources, 0);
    assert.strictEqual(h.outputDisposals, 1);
    assert.strictEqual(h.commands.size, 0);
    assert.strictEqual(h.configurationHandlers.size, 0);
    assert.strictEqual(h.formatters.size, 0);
  });

  test("accepted A -> B change rebuilds options and preserves extension handlers", async () => {
    await h.activate();
    const provider = [...h.formatters][0];
    const listener = [...h.configurationHandlers][0];
    const command = h.commands.get("ktav.restartServer");
    const subscriptions = h.context.subscriptions.length;
    const changed = h.change("server-B");
    h.prompts[0].resolve("Restart");
    await changed;

    assert.deepStrictEqual(h.clients.map((c) => c.command), ["server-A", "server-B"]);
    assert.strictEqual(h.clients[0].disposeCalls, 1);
    assert.strictEqual(h.clients[0].restartCalls, 0);
    assert.deepStrictEqual(h.clients[0].stopTimeouts, [2000]);
    assert.strictEqual(h.clients[1].serverOptions.debug.command, "server-B");
    assert.strictEqual(h.clients[1].clientOptions.outputChannel, h.output);
    assert.strictEqual(h.clients[1].clientOptions.traceOutputChannel, h.output);
    assert.strictEqual(h.clients[1].clientOptions.synchronize.configurationSection, "ktav");
    assert.strictEqual(h.context.subscriptions.length, subscriptions);
    assert.strictEqual([...h.formatters][0], provider);
    assert.strictEqual([...h.configurationHandlers][0], listener);
    assert.strictEqual(h.commands.get("ktav.restartServer"), command);
    assert.strictEqual(h.outputDisposals, 0);
    assert.strictEqual(h.maxActiveResources, 1);
    await h.format();
    assert.strictEqual(h.events.at(-1), "format:server-B");

    const again = h.change("server-C");
    h.prompts[1].resolve("Restart");
    await again;
    await h.format();
    assert.strictEqual(h.events.at(-1), "format:server-C");
    assert.strictEqual(h.clients[1].disposeCalls, 1);
  });

  test("dismissed prompt keeps A; the palette command then discovers B", async () => {
    await h.activate();
    const changed = h.change("server-B");
    h.prompts[0].resolve(undefined);
    await changed;
    assert.strictEqual(h.clients.length, 1);
    assert.strictEqual(h.clients[0].disposeCalls, 0);
    await h.format();
    assert.strictEqual(h.events.at(-1), "format:server-A");
    await h.restart();
    assert.strictEqual(h.clients[1].command, "server-B");
  });

  test("invalid explicit path rediscoveries use bundled and PATH fallbacks", async () => {
    await h.activate();
    h.existing.add(h.bundled);
    const bundled = h.change("missing-server");
    h.prompts[0].resolve("Restart");
    await bundled;
    assert.strictEqual(h.clients[1].command, h.bundled);
    h.existing.delete(h.bundled);
    const fallback = h.change("");
    h.prompts[1].resolve("Restart");
    await fallback;
    assert.strictEqual(h.clients[2].command, "ktav-lsp");
    assert.strictEqual(h.errors.length, 0);
  });

  test("failed replacement reports an error and a later valid change recovers", async () => {
    await h.activate();
    h.failCommands.add("ktav-lsp");
    const invalid = h.change("missing-server");
    h.prompts[0].resolve("Restart");
    await invalid;
    assert.strictEqual(h.clients[0].disposeCalls, 1);
    assert.strictEqual(h.clients[1].command, "ktav-lsp");
    assert.strictEqual(h.activeResources, 0);
    assert.match(h.errors[0], /restart failed: spawn failed: ktav-lsp/);
    const recovered = h.change("server-B");
    h.prompts[1].resolve("Restart");
    await recovered;
    assert.strictEqual(h.clients[1].disposeCalls, 1);
    assert.strictEqual(h.clients[2].command, "server-B");
    assert.strictEqual(h.clients[2].running, true);
    assert.strictEqual(h.diagnosticResources, 1);
  });

  test("failed initial startup retains handlers for recovery", async () => {
    h.failCommands.add("server-A");
    await h.activate();
    assert.match(h.errors[0], /failed to start language server "server-A"/);
    const recovered = h.change("server-B");
    h.prompts[0].resolve("Restart");
    await recovered;
    assert.strictEqual(h.clients[0].disposeCalls, 1);
    assert.strictEqual(h.clients[1].running, true);
    assert.strictEqual(h.diagnosticResources, 1);
  });

  test("configuration changes during initial startup are not lost", async () => {
    const start = deferred<void>();
    h.startGates.set("server-A", start);
    const activation = h.activate();
    try {
      const initial = await h.clientCreated(0);
      await initial.startEntered.promise;
      const changed = h.change("server-B");
      h.prompts[0].resolve("Restart");
      start.resolve();
      await Promise.all([activation, changed]);
      assert.deepStrictEqual(h.clients.map((c) => c.command), ["server-A", "server-B"]);
      assert.strictEqual(initial.disposeCalls, 1);
      assert.strictEqual(h.maxActiveResources, 1);
    } finally {
      start.resolve();
    }
  });

  test("a failed stop cannot start a second server or poison the restart queue", async () => {
    await h.activate();
    h.clients[0].stopFailure = new Error("stop failed");
    const changed = h.change("server-B");
    h.prompts[0].resolve("Restart");
    await changed;
    assert.strictEqual(h.clients.length, 1);
    assert.match(h.errors[0], /restart failed: stop failed/);
    await h.restart();
    assert.strictEqual(h.clients[1].command, "server-B");
    assert.strictEqual(h.maxActiveResources, 1);
  });

  test("concurrent prompt and palette restarts wait for stop and start", async () => {
    await h.activate();
    const stop = deferred<void>();
    const start = deferred<void>();
    h.clients[0].stopGate = stop;
    h.startGates.set("server-B", start);
    try {
      const changed = h.change("server-B");
      h.prompts[0].resolve("Restart");
      await h.clients[0].stopEntered.promise;
      const manual = h.restart();
      assert.strictEqual(h.clients.length, 1);
      await h.format();
      assert.ok(!h.events.some((event) => event.startsWith("format:")));
      stop.resolve();
      const replacement = await h.clientCreated(1);
      await replacement.startEntered.promise;
      assert.strictEqual(h.clients.length, 2);
      assert.strictEqual(replacement.disposeCalls, 0);
      start.resolve();
      await Promise.all([changed, manual]);
      assert.deepStrictEqual(h.clients.map((c) => c.command), ["server-A", "server-B", "server-B"]);
      assert.strictEqual(replacement.disposeCalls, 1);
      assert.strictEqual(h.maxActiveResources, 1);
      assert.ok(h.events.indexOf("stopped:server-A") < h.events.indexOf("create:server-B"));
      assert.ok(h.events.indexOf("stopped:server-B") < h.events.lastIndexOf("create:server-B"));
    } finally {
      stop.resolve();
      start.resolve();
    }
  });

  test("older prompt cannot override a newer cancelled configuration change", async () => {
    await h.activate();
    const older = h.change("server-B");
    const newer = h.change("server-C");
    h.prompts[1].resolve(undefined);
    await newer;
    h.prompts[0].resolve("Restart");
    await older;
    assert.strictEqual(h.clients.length, 1);
    assert.strictEqual(h.clients[0].disposeCalls, 0);
  });

  test("unrelated configuration changes do not prompt or replace the client", async () => {
    await h.activate();
    await h.change("server-B", "ktav.trace.server");
    assert.strictEqual(h.prompts.length, 0);
    assert.strictEqual(h.clients.length, 1);
  });

  test("deactivation prevents a replacement after an in-flight stop", async () => {
    await h.activate();
    const stop = deferred<void>();
    h.clients[0].stopGate = stop;
    try {
      const changed = h.change("server-B");
      h.prompts[0].resolve("Restart");
      await h.clients[0].stopEntered.promise;
      const shutdown = h.deactivate();
      const manual = h.restart();
      stop.resolve();
      await Promise.all([changed, shutdown, manual]);
      assert.strictEqual(h.clients.length, 1);
      assert.strictEqual(h.clients[0].disposeCalls, 1);
    } finally {
      stop.resolve();
    }
  });

  test("deactivation disposes an in-flight replacement and skips queued restarts", async () => {
    await h.activate();
    const start = deferred<void>();
    h.startGates.set("server-B", start);
    const changed = h.change("server-B");
    h.prompts[0].resolve("Restart");
    try {
      const replacement = await h.clientCreated(1);
      await replacement.startEntered.promise;
      const manual = h.restart();
      const shutdown = h.deactivate();
      start.resolve();
      await Promise.all([changed, manual, shutdown]);
      assert.strictEqual(h.clients.length, 2);
      assert.strictEqual(replacement.disposeCalls, 1);
      assert.strictEqual(h.activeResources, 0);
    } finally {
      start.resolve();
    }
  });

  test("deactivation waits for startup and ignores a late prompt response", async () => {
    const start = deferred<void>();
    h.startGates.set("server-A", start);
    const activation = h.activate();
    try {
      const manual = h.restart();
      const initial = await h.clientCreated(0);
      await initial.startEntered.promise;
      const changed = h.change("server-B");
      const shutdown = h.deactivate();
      start.resolve();
      await Promise.all([activation, manual, shutdown]);
      h.prompts[0].resolve("Restart");
      await changed;
      assert.strictEqual(h.clients.length, 1);
      assert.strictEqual(h.clients[0].disposeCalls, 1);
      assert.deepStrictEqual(h.clients[0].stopTimeouts, [2000]);
    } finally {
      start.resolve();
    }
  });
});
