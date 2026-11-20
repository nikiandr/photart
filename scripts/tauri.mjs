import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);

if (
  args[0] === "dev" &&
  !args.some((arg) => arg === "--config" || arg.startsWith("--config="))
) {
  args.push("--config", "src-tauri/tauri.dev.conf.json");
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const command = path.join(
  root,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "tauri.cmd" : "tauri",
);

const child = spawn(command, args, { stdio: "inherit" });

for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, () => {
    if (child.exitCode === null && !child.killed) {
      child.kill(signal);
    }
  });
}

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 1);
});
