#!/usr/bin/env node
// Rebuilds this repository's generated Markdown from the root-docs/ unit
// trees, using @ktav-lang/polydoc: the root README/CHANGELOG/CONTRIBUTING,
// the lsp/, vscode/ and intellij/ README+CHANGELOG pairs, historical
// lsp/intellij notes, and grammars/README — each with Russian and Chinese
// translations. Run
// with --check for a CI-friendly, read-only verification instead of
// regenerating the files.
//
// Each documentation set configures polydoc separately (re-calling
// configure() between independent builds in one process is supported) and
// keeps its unit tree in `<setRoot>/root-docs/<DOC>/`. Generated files
// keep their historical paths — translations under docs/i18n/ for the
// root set and under <component>/docs/ for the component sets — so the
// per-set path map below, not writeRootDocs, decides where output goes.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { configure, buildRootDocs } from '@ktav-lang/polydoc';

const LANGS = ['en', 'ru', 'zh'];

const SETS = [
  {
    name: 'root',
    root: '.',
    docs: {
      README: {
        en: 'README.md',
        ru: 'docs/i18n/README.ru.md',
        zh: 'docs/i18n/README.zh.md',
      },
      CHANGELOG: {
        en: 'CHANGELOG.md',
        ru: 'docs/i18n/CHANGELOG.ru.md',
        zh: 'docs/i18n/CHANGELOG.zh.md',
      },
      CONTRIBUTING: {
        en: 'CONTRIBUTING.md',
        ru: 'docs/i18n/CONTRIBUTING.ru.md',
        zh: 'docs/i18n/CONTRIBUTING.zh.md',
      },
    },
  },
  {
    name: 'lsp',
    root: 'lsp',
    docs: {
      README: {
        en: 'lsp/README.md',
        ru: 'lsp/docs/README.ru.md',
        zh: 'lsp/docs/README.zh.md',
      },
      CHANGELOG: {
        en: 'lsp/CHANGELOG.md',
        ru: 'lsp/docs/CHANGELOG.ru.md',
        zh: 'lsp/docs/CHANGELOG.zh.md',
      },
      'bench-baseline': {
        en: 'lsp/docs/bench-baseline.md',
        ru: 'lsp/docs/bench-baseline.ru.md',
        zh: 'lsp/docs/bench-baseline.zh.md',
      },
    },
  },
  {
    name: 'vscode',
    root: 'vscode',
    docs: {
      README: {
        en: 'vscode/README.md',
        ru: 'vscode/docs/README.ru.md',
        zh: 'vscode/docs/README.zh.md',
      },
      CHANGELOG: {
        en: 'vscode/CHANGELOG.md',
        ru: 'vscode/docs/CHANGELOG.ru.md',
        zh: 'vscode/docs/CHANGELOG.zh.md',
      },
    },
  },
  {
    name: 'intellij',
    root: 'intellij',
    docs: {
      README: {
        en: 'intellij/README.md',
        ru: 'intellij/docs/README.ru.md',
        zh: 'intellij/docs/README.zh.md',
      },
      CHANGELOG: {
        en: 'intellij/CHANGELOG.md',
        ru: 'intellij/docs/CHANGELOG.ru.md',
        zh: 'intellij/docs/CHANGELOG.zh.md',
      },
      TEXTMATE_REGISTRATION_PROBLEM: {
        en: 'intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.md',
        ru: 'intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.ru.md',
        zh: 'intellij/docs/TEXTMATE_REGISTRATION_PROBLEM.zh.md',
      },
    },
  },
  {
    name: 'grammars',
    root: 'grammars',
    docs: {
      README: {
        en: 'grammars/README.md',
        ru: 'grammars/docs/README.ru.md',
        zh: 'grammars/docs/README.zh.md',
      },
    },
  },
  {
    // Per-editor setup notes under docs/. English keeps its historical
    // path; translations live under docs/i18n/editors/<lang>/ rather than
    // flat in docs/i18n/ to keep that directory's entry count small.
    name: 'docs',
    root: '.',
    docs: {
      emacs: {
        en: 'docs/emacs.md',
        ru: 'docs/i18n/editors/ru/emacs.md',
        zh: 'docs/i18n/editors/zh/emacs.md',
      },
      helix: {
        en: 'docs/helix.md',
        ru: 'docs/i18n/editors/ru/helix.md',
        zh: 'docs/i18n/editors/zh/helix.md',
      },
      neovim: {
        en: 'docs/neovim.md',
        ru: 'docs/i18n/editors/ru/neovim.md',
        zh: 'docs/i18n/editors/zh/neovim.md',
      },
      sublime: {
        en: 'docs/sublime.md',
        ru: 'docs/i18n/editors/ru/sublime.md',
        zh: 'docs/i18n/editors/zh/sublime.md',
      },
      zed: {
        en: 'docs/zed.md',
        ru: 'docs/i18n/editors/ru/zed.md',
        zh: 'docs/i18n/editors/zh/zed.md',
      },
    },
  },
];

// Per-unit validation proves every meaning has every language. It does
// NOT prove the languages describe the same DOCUMENT: a heading demoted
// from ## to ### in one translation, or an extra heading in another,
// passes unit validation untouched. This check catches that class of
// drift.
function headingSkeleton(markdown) {
  const levels = [];
  let fenceChar = null;
  let fenceLen = 0;
  for (const line of markdown.split('\n')) {
    const fence = line.match(/^\s{0,3}(`{3,}|~{3,})/u);
    if (fence) {
      const char = fence[1][0];
      const len = fence[1].length;
      if (fenceChar === null) { fenceChar = char; fenceLen = len; }
      else if (char === fenceChar && len >= fenceLen) { fenceChar = null; }
      continue;
    }
    if (fenceChar !== null) continue;
    const heading = line.match(/^(#{1,6})\s+\S/u);
    if (heading) levels.push(heading[1].length);
  }
  return levels;
}

function structuralProblems(label, perLang) {
  const [reference, ...others] = LANGS;
  const base = headingSkeleton(perLang.get(reference).toString('utf8'));
  const problems = [];
  for (const lang of others) {
    const other = headingSkeleton(perLang.get(lang).toString('utf8'));
    if (other.length !== base.length) {
      problems.push(`${label}: ${lang} has ${other.length} heading(s) but ${reference} has ` +
        `${base.length} — the translations describe different documents`);
      continue;
    }
    const at = base.findIndex((level, i) => level !== other[i]);
    if (at !== -1) {
      problems.push(`${label}: heading #${at + 1} is level ${other[at]} in ${lang} but ` +
        `level ${base[at]} in ${reference}`);
    }
  }
  return problems;
}

function unitsDirLabel(set, doc) {
  const prefix = set.root === '.' ? '' : `${set.root}/`;
  return `${prefix}root-docs/${doc}`;
}

function usage() {
  process.stderr.write(
    'usage: node scripts/build-docs.mjs [--check]\n' +
    '  (no args)  regenerate every generated .md (en/ru/zh, six documentation sets)\n' +
    '  --check    verify the generated files match the root-docs/ unit trees without writing\n'
  );
}

function cli() {
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const repoRoot = path.resolve(scriptDir, '..', '..');

  const args = process.argv.slice(2);
  if (args.includes('-h') || args.includes('--help')) { usage(); process.exit(0); }
  if (args.length > 1 || (args.length === 1 && args[0] !== '--check')) {
    usage();
    process.exit(1);
  }
  const checkMode = args[0] === '--check';

  let built;
  try {
    built = SETS.map((set) => {
      configure({ langs: LANGS, rootDocuments: Object.keys(set.docs) });
      const docs = buildRootDocs(path.resolve(repoRoot, set.root));
      for (const [name, perLang] of docs) {
        const problems = structuralProblems(`${set.name}/${name}`, perLang);
        if (problems.length > 0) {
          throw new Error(problems.join('\n'));
        }
      }
      return { set, docs };
    });
  } catch (e) {
    process.stderr.write(`build-docs: ${e.message}\n`);
    process.exit(1);
  }

  let count = 0;
  if (!checkMode) {
    for (const { set, docs } of built) {
      for (const [name, perLang] of docs) {
        for (const lang of LANGS) {
          const target = path.join(repoRoot, set.docs[name][lang]);
          fs.writeFileSync(target, perLang.get(lang));
          count++;
        }
      }
    }
    process.stdout.write(`build-docs: assembled ${count} document(s) from ` +
      `${SETS.length} root-docs/ set(s)\n`);
    process.exit(0);
  }

  const problems = [];
  for (const { set, docs } of built) {
    for (const [name, perLang] of docs) {
      for (const lang of LANGS) {
        const rel = set.docs[name][lang];
        const unitsLabel = `${unitsDirLabel(set, name)}/`;
        const expected = perLang.get(lang);
        let actual;
        try {
          actual = fs.readFileSync(path.join(repoRoot, rel));
        } catch (e) {
          problems.push(`${rel} is missing or unreadable (${e.message}); it is generated from ` +
            `${unitsLabel}`);
          continue;
        }
        if (actual.equals(expected)) continue;
        let off = 0;
        const min = Math.min(actual.length, expected.length);
        while (off < min && actual[off] === expected[off]) off++;
        const line = expected.subarray(0, off).toString('utf8').split('\n').length;
        problems.push(
          `${rel} differs from what ${unitsLabel} generates, first at byte ${off} ` +
          `(line ${line}); edit the unit source, never the generated file`);
      }
    }
  }
  if (problems.length > 0) {
    for (const problem of problems) {
      process.stderr.write(`build-docs --check: ${problem}\n`);
    }
    process.exit(1);
  }
  process.exit(0);
}

const isMain = process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (isMain) cli();
