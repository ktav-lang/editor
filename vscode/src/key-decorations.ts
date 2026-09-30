// Editor decorations driven by the server's semantic tokens:
// - a soft underline under every key (theme colour `ktav.keyUnderline`,
//   so it can be lighter than the key text itself);
// - the paired quote of a quoted key, highlighted like a matching bracket.
import * as vscode from "vscode";
import type { LanguageClient } from "vscode-languageclient/node";
import { decodeKeyTokens, KeyTokens, pairAtCursor } from "./quote-pairs";

interface Cached {
  version: number;
  tokens: KeyTokens;
}

const EMPTY: KeyTokens = { keys: [], pairs: [] };

/** Registers the decorations; returns a refresh to call once the server is up. */
export function registerKeyDecorations(
  context: vscode.ExtensionContext,
  getClient: () => LanguageClient | undefined,
): () => void {
  const underline = vscode.window.createTextEditorDecorationType({
    borderColor: new vscode.ThemeColor("ktav.keyUnderline"),
    borderStyle: "solid",
    borderWidth: "0 0 1px 0",
  });
  const matchedQuote = vscode.window.createTextEditorDecorationType({
    backgroundColor: new vscode.ThemeColor("editorBracketMatch.background"),
    borderColor: new vscode.ThemeColor("editorBracketMatch.border"),
    borderStyle: "solid",
    borderWidth: "1px",
  });
  const cache = new Map<string, Cached>();
  const timers = new Map<string, ReturnType<typeof setTimeout>>();

  async function tokensFor(document: vscode.TextDocument): Promise<KeyTokens> {
    const key = document.uri.toString();
    const hit = cache.get(key);
    if (hit && hit.version === document.version) return hit.tokens;
    const client = getClient();
    const legend = client?.initializeResult?.capabilities.semanticTokensProvider?.legend;
    if (!client || !client.isRunning() || !legend) return EMPTY;
    const version = document.version;
    const result = await client.sendRequest<{ data: number[] } | null>(
      "textDocument/semanticTokens/full",
      { textDocument: { uri: key } },
    );
    const tokens = decodeKeyTokens(result?.data ?? [], legend, (line) =>
      line < document.lineCount ? document.lineAt(line).text : undefined);
    if (document.version === version) cache.set(key, { version, tokens });
    return tokens;
  }

  async function refresh(editor: vscode.TextEditor | undefined, underlineToo: boolean): Promise<void> {
    if (!editor || editor.document.languageId !== "ktav") return;
    const document = editor.document;
    const version = document.version;
    let tokens = EMPTY;
    try {
      tokens = await tokensFor(document);
    } catch {
      // Server restarting or request cancelled: keep the editor undecorated.
    }
    if (document.version !== version) return;
    if (underlineToo) {
      editor.setDecorations(underline, tokens.keys.map((k) => new vscode.Range(k.line, k.start, k.line, k.end)));
    }
    const pos = editor.selection.active;
    const pair = editor.selection.isEmpty ? pairAtCursor(tokens.pairs, pos.line, pos.character) : undefined;
    editor.setDecorations(matchedQuote, pair ? [
      new vscode.Range(pair.line, pair.open, pair.line, pair.open + 1),
      new vscode.Range(pair.line, pair.close, pair.line, pair.close + 1),
    ] : []);
  }

  function refreshDocument(document: vscode.TextDocument): void {
    const key = document.uri.toString();
    clearTimeout(timers.get(key));
    // Let the server see the edit before asking for tokens again.
    timers.set(key, setTimeout(() => {
      timers.delete(key);
      for (const editor of vscode.window.visibleTextEditors) {
        if (editor.document === document) void refresh(editor, true);
      }
    }, 150));
  }

  context.subscriptions.push(
    underline,
    matchedQuote,
    { dispose: () => timers.forEach(clearTimeout) },
    vscode.window.onDidChangeTextEditorSelection((e) => void refresh(e.textEditor, false)),
    vscode.window.onDidChangeVisibleTextEditors((editors) => editors.forEach((e) => void refresh(e, true))),
    vscode.workspace.onDidChangeTextDocument((e) => {
      if (e.document.languageId !== "ktav") return;
      cache.delete(e.document.uri.toString());
      refreshDocument(e.document);
    }),
    vscode.workspace.onDidCloseTextDocument((d) => cache.delete(d.uri.toString())),
  );
  return () => {
    cache.clear();
    vscode.window.visibleTextEditors.forEach((e) => void refresh(e, true));
  };
}

