// VS Code can leave its non-ASCII boxes on a Ktav editor restored at window
// start: the unicode highlighter built before the `[ktav]` defaults apply
// paints a late worker result after it is disposed (microsoft/vscode#338961).
// Re-opening the editor in place, as a tab switch does, drops those boxes.
import type * as vscode from "vscode";

/** Editors visible when a fresh extension host starts were restored. */
export const FRESH_HOST_SECONDS = 60;
/** Lets the stale worker result land before the editor is re-opened. */
export const SETTLE_MS = 1500;
/** `EditorActivation.PRESERVE`: keep the active group and focus. */
const PRESERVE = 3;

export interface Api {
  window: Pick<typeof vscode.window, "visibleTextEditors" | "tabGroups">;
  workspace: Pick<typeof vscode.workspace, "isTrusted">;
  commands: Pick<typeof vscode.commands, "executeCommand">;
  TabInputText: typeof vscode.TabInputText;
}

/** A plain Ktav text tab with non-ASCII text, active in its group. */
export function needsReattach(api: Api, editor: vscode.TextEditor): boolean {
  const doc = editor.document;
  if (doc.languageId !== "ktav" || doc.uri.scheme === "untitled" || !/[^\x00-\x7f]/.test(doc.getText())) {
    return false;
  }
  // Diff sides and notebook cells are visible editors too; only a text tab
  // may be re-opened by URI without changing what the group shows.
  const input = api.window.tabGroups.all.find((g) => g.viewColumn === editor.viewColumn)?.activeTab?.input;
  return input instanceof api.TabInputText && input.uri.toString() === doc.uri.toString();
}

export function reattachRestoredEditors(
  api: Api,
  uptimeSeconds: number,
  schedule: (run: () => void, ms: number) => unknown = setTimeout,
): void {
  // Only Restricted Mode boxes every non-ASCII character before the defaults
  // apply; re-opening also replaces the TextEditor other code may hold.
  if (api.workspace.isTrusted || uptimeSeconds > FRESH_HOST_SECONDS) return;
  const restored = api.window.visibleTextEditors.filter((e) => needsReattach(api, e));
  if (restored.length === 0) return;
  schedule(() => {
    for (const editor of restored) {
      if (!api.window.visibleTextEditors.includes(editor) || !needsReattach(api, editor)) continue;
      // `vscode.open` runs `_workbench.open`, which passes `forceReload`
      // through: the group re-sets the same input on the same editor.
      Promise.resolve(api.commands.executeCommand("_workbench.open", editor.document.uri, [
        editor.viewColumn! - 1,
        { forceReload: true, preserveFocus: true, activation: PRESERVE },
      ])).catch(() => {
        // Internal command changed: the boxes stay until a tab switch.
      });
    }
  }, SETTLE_MS);
}
