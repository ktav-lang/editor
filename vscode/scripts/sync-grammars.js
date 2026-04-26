#!/usr/bin/env node
/**
 * sync-grammars.js
 *
 * Copies the shared TextMate grammar and language-configuration files from
 * the monorepo's `../grammars/` directory into the VS Code extension package
 * so that `vsce package` can include them.
 *
 * The source-of-truth lives at `editor/grammars/`; this script just mirrors
 * those files locally. Re-run it whenever the upstream grammar changes.
 */

"use strict";

const fs = require("fs");
const path = require("path");

const here = __dirname;
const pkgRoot = path.resolve(here, "..");
const grammarsRoot = path.resolve(pkgRoot, "..", "grammars");

const targets = [
  {
    src: path.join(grammarsRoot, "ktav.tmLanguage.json"),
    dst: path.join(pkgRoot, "syntaxes", "ktav.tmLanguage.json"),
  },
  {
    src: path.join(grammarsRoot, "language-configuration.json"),
    dst: path.join(pkgRoot, "language-configuration.json"),
  },
];

function main() {
  for (const { src, dst } of targets) {
    if (!fs.existsSync(src)) {
      console.error(`sync-grammars: source missing: ${src}`);
      process.exit(1);
    }
    fs.mkdirSync(path.dirname(dst), { recursive: true });
    fs.copyFileSync(src, dst);
    console.log(
      `sync-grammars: ${path.relative(pkgRoot, src)} -> ${path.relative(pkgRoot, dst)}`,
    );
  }
}

main();
