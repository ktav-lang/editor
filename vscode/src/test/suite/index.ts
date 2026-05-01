import * as path from "path";
import * as fs from "fs";
import Mocha from "mocha";

export function run(): Promise<void> {
  const mocha = new Mocha({ ui: "tdd", color: true, timeout: 60_000 });
  const testsRoot = __dirname;

  return new Promise((resolve, reject) => {
    try {
      for (const f of fs.readdirSync(testsRoot)) {
        if (f.endsWith(".test.js")) {
          mocha.addFile(path.join(testsRoot, f));
        }
      }
      mocha.run((failures) => {
        if (failures > 0) {
          reject(new Error(`${failures} test(s) failed.`));
        } else {
          resolve();
        }
      });
    } catch (err) {
      reject(err);
    }
  });
}
