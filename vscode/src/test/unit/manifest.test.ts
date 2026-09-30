import * as assert from "assert";
import * as fs from "fs";
import * as path from "path";

// Manifest contracts the editor experience depends on; no VS Code runtime.
const manifest = JSON.parse(fs.readFileSync(path.resolve(__dirname, "..", "..", "..", "package.json"), "utf8"));
const contributes = manifest.contributes;
const defaults = contributes.configurationDefaults;

suite("extension manifest", () => {
  test("works in Restricted Mode without trusting a workspace server path", () => {
    const untrusted = manifest.capabilities.untrustedWorkspaces;
    assert.strictEqual(untrusted.supported, "limited");
    assert.deepStrictEqual(untrusted.restrictedConfigurations, ["ktav.server.path"]);
  });

  test("Ktav files do not box non-ASCII text such as Cyrillic keys or §", () => {
    assert.strictEqual(defaults["[ktav]"]["editor.unicodeHighlight.nonBasicASCII"], false);
    assert.strictEqual(defaults["[ktav]"]["editor.unicodeHighlight.ambiguousCharacters"], false);
    assert.strictEqual(defaults["[ktav]"]["editor.unicodeHighlight.invisibleCharacters"], undefined);
  });

  test("quoted keys have a semantic modifier and their own colours", () => {
    assert.ok(contributes.semanticTokenModifiers.some((m: { id: string }) => m.id === "quoted"));
    const rules = defaults["editor.semanticTokenColorCustomizations"].rules;
    assert.ok(rules["property.quoted:ktav"], "quoted key content colour");
    assert.notStrictEqual(rules["property:ktav"], "#333333", "bare keys must stand out from text");
    const scopes = defaults["editor.tokenColorCustomizations"].textMateRules.flatMap((r: { scope: string[] }) => r.scope);
    for (const scope of [
      "comment.line.number-sign.ktav",
      "string.quoted.double.key.ktav",
      "punctuation.definition.string.begin.ktav",
      "punctuation.definition.string.end.ktav",
    ]) {
      assert.ok(scopes.includes(scope), `missing colour rule for ${scope}`);
    }
    const colourOf = (scope: string) => defaults["editor.tokenColorCustomizations"].textMateRules
      .find((r: { scope: string[] }) => r.scope.includes(scope)).settings.foreground;
    assert.notStrictEqual(
      colourOf("punctuation.definition.string.begin.ktav"),
      colourOf("string.quoted.double.key.ktav"),
      "the paired quotes must be coloured apart from the key text",
    );
  });

  test("keys get a soft, theme-able underline instead of an underlined font", () => {
    const colour = contributes.colors.find((c: { id: string }) => c.id === "ktav.keyUnderline");
    assert.ok(colour, "ktav.keyUnderline colour contribution");
    assert.match(colour.defaults.light, /^#[0-9A-F]{8}$/i, "translucent light default");
    const rules = defaults["editor.tokenColorCustomizations"].textMateRules;
    assert.ok(rules.every((r: { settings: { fontStyle?: string } }) => !/underline/.test(r.settings.fontStyle ?? "")));
  });

  test("word selection and occurrence highlight stop at key quotes", () => {
    const config = JSON.parse(fs.readFileSync(path.resolve(__dirname, "..", "..", "..", "syntaxes", "language-configuration.json"), "utf8"));
    const words = '"ключ в кавычках": да, \'x\': `y`'.match(new RegExp(config.wordPattern, "g"));
    assert.deepStrictEqual(words, ["ключ", "в", "кавычках", "да", "x", "y"]);
  });
});
