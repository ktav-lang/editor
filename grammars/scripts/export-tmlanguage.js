#!/usr/bin/env node
/**
 * Exports the canonical JSON TextMate grammar as an XML .tmLanguage plist,
 * which Sublime Text can load without additional conversion dependencies.
 * Usage (from the editor repository):
 *   node grammars/scripts/export-tmlanguage.js /path/to/ktav.tmLanguage
 */
"use strict";

const fs = require("fs");
const path = require("path");

function escapeXml(value) {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function plist(value, indent) {
  if (typeof value === "string") {
    return `${indent}<string>${escapeXml(value)}</string>`;
  }
  if (Array.isArray(value)) {
    return [
      `${indent}<array>`,
      ...value.map((entry) => plist(entry, `${indent}  `)),
      `${indent}</array>`,
    ].join("\n");
  }
  if (value && typeof value === "object") {
    return [
      `${indent}<dict>`,
      ...Object.entries(value).flatMap(([key, entry]) => [
        `${indent}  <key>${escapeXml(key)}</key>`,
        plist(entry, `${indent}  `),
      ]),
      `${indent}</dict>`,
    ].join("\n");
  }
  throw new Error(`Unsupported grammar plist value: ${JSON.stringify(value)}`);
}

function main() {
  if (process.argv.length !== 3 || path.extname(process.argv[2]) !== ".tmLanguage") {
    console.error("Usage: node grammars/scripts/export-tmlanguage.js /path/to/ktav.tmLanguage");
    process.exit(1);
  }
  const source = path.resolve(__dirname, "..", "ktav.tmLanguage.json");
  const target = path.resolve(process.argv[2]);
  const grammar = JSON.parse(fs.readFileSync(source, "utf8"));
  delete grammar.$schema;
  const xml = [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">',
    '<plist version="1.0">',
    plist(grammar, "  "),
    "</plist>",
    "",
  ].join("\n");
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, xml, "utf8");
  console.log(`export-tmlanguage: ${source} -> ${target} (source.ktav)`);
}

main();
