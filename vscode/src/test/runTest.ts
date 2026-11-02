import * as net from "net";
import * as path from "path";
import { runTests } from "@vscode/test-electron";

async function main(): Promise<void> {
  try {
    // Не прерывать IPv4 handshake через стандартные 250 ms при недоступном IPv6.
    net.setDefaultAutoSelectFamilyAttemptTimeout(2_500);
    // out/test/runTest.js -> extension root is two levels up from compiled file.
    const extensionDevelopmentPath = path.resolve(__dirname, "..", "..");
    const extensionTestsPath = path.resolve(__dirname, "./suite/index");

    await runTests({
      extensionDevelopmentPath,
      extensionTestsPath,
      // No GPU under xvfb: software rendering can stall the window long
      // enough for VS Code to report it unresponsive and abort the run.
      launchArgs: ["--disable-extensions", "--disable-gpu", "--disable-dev-shm-usage"],
    });
  } catch (err) {
    console.error("Failed to run tests:", err);
    process.exit(1);
  }
}

void main();
