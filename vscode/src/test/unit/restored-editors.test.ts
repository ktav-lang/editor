import * as assert from "assert";
import type * as vscode from "vscode";
import { Api, FRESH_HOST_SECONDS, reattachRestoredEditors, SETTLE_MS } from "../../restored-editors";

class FakeTabInputText {
  constructor(readonly uri: { toString(): string }) {}
}
class FakeTabInputTextDiff {
  constructor(readonly original: unknown, readonly modified: unknown) {}
}

function uri(text: string, scheme = "file") {
  return { scheme, toString: () => `${scheme}:${text}` };
}

function editor(name: string, text: string, viewColumn = 1, languageId = "ktav", scheme = "file") {
  return { document: { languageId, uri: uri(name, scheme), getText: () => text }, viewColumn } as unknown as vscode.TextEditor;
}

function harness(editors: vscode.TextEditor[], tabInput?: (e: vscode.TextEditor) => unknown) {
  const calls: unknown[][] = [];
  const scheduled: { run: () => void; ms: number }[] = [];
  const window = {
    visibleTextEditors: editors,
    tabGroups: {
      all: editors.map((e) => ({
        viewColumn: e.viewColumn,
        activeTab: { input: tabInput ? tabInput(e) : new FakeTabInputText(e.document.uri) },
      })),
    },
  };
  const api = {
    window,
    workspace: { isTrusted: false },
    commands: { executeCommand: (...args: unknown[]) => { calls.push(args); return Promise.resolve(); } },
    TabInputText: FakeTabInputText,
  } as unknown as Api;
  const run = (uptime = 5) => reattachRestoredEditors(api, uptime, (fn, ms) => scheduled.push({ run: fn, ms }));
  return { api, window, calls, scheduled, run };
}

suite("re-open restored Ktav editors in place", () => {
  test("a restored Ktav tab with non-ASCII text is re-opened with forceReload after settling", () => {
    const ed = editor("a.ktav", "ключ: значение §", 2);
    const h = harness([ed]);
    h.run();
    assert.strictEqual(h.scheduled.length, 1);
    assert.strictEqual(h.scheduled[0].ms, SETTLE_MS);
    assert.strictEqual(h.calls.length, 0, "nothing happens before the settle delay");
    h.scheduled[0].run();
    assert.deepStrictEqual(h.calls, [[
      "_workbench.open", ed.document.uri,
      [1, { forceReload: true, preserveFocus: true, activation: 3 }],
    ]]);
  });

  test("editors that cannot carry stale boxes, or tabs other than plain text, are left alone", () => {
    const cases: [string, ReturnType<typeof harness>][] = [
      ["ASCII only", harness([editor("a.ktav", "key: value")])],
      ["other language", harness([editor("a.md", "ключ", 1, "markdown")])],
      ["untitled", harness([editor("Untitled-1", "ключ", 1, "ktav", "untitled")])],
      ["diff side", harness([editor("a.ktav", "ключ")], () => new FakeTabInputTextDiff(1, 2))],
      ["other tab active", harness([editor("a.ktav", "ключ")], () => new FakeTabInputText(uri("b.ktav")))],
    ];
    for (const [name, h] of cases) {
      h.run();
      assert.strictEqual(h.scheduled.length, 0, name);
    }
  });

  test("an old extension host restored nothing", () => {
    const h = harness([editor("a.ktav", "ключ")]);
    h.run(FRESH_HOST_SECONDS + 1);
    assert.strictEqual(h.scheduled.length, 0);
  });

  test("a trusted workspace never boxes Cyrillic, so its editors are left alone", () => {
    const h = harness([editor("a.ktav", "ключ: значение §")]);
    (h.api.workspace as { isTrusted: boolean }).isTrusted = true;
    h.run();
    assert.strictEqual(h.scheduled.length, 0);
  });

  test("an editor hidden before the delay ends, like a tab switch, is not re-opened", () => {
    const ed = editor("a.ktav", "ключ");
    const h = harness([ed]);
    h.run();
    h.window.visibleTextEditors = [];
    h.scheduled[0].run();
    assert.deepStrictEqual(h.calls, []);
  });

  test("a failing internal command is swallowed", async () => {
    const h = harness([editor("a.ktav", "ключ")]);
    h.api.commands.executeCommand = (() => Promise.reject(new Error("gone"))) as never;
    h.run();
    h.scheduled[0].run();
    await new Promise((done) => setImmediate(done));
  });
});
