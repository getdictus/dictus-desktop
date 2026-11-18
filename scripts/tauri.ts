// scripts/tauri.ts — `bun tauri …` entry point (package.json "tauri" script).
//
// Dev and debug builds run under their own identity, "Dictus Dev"
// (`com.dictus.desktop.dev`), so they never read or write the installed app's
// data and can run next to it (issue #70). This wrapper merges
// src-tauri/tauri.dev.conf.json for:
//   - `tauri dev`
//   - `tauri build --debug`
// Every other command (release `tauri build`, `tauri icon`, …) is passed
// through unchanged and keeps `com.dictus.desktop`.
//
// An explicit `--config`/`-c` on the command line wins: nothing is injected.

import { fileURLToPath } from "node:url";

const DEV_CONFIG = fileURLToPath(
  new URL("../src-tauri/tauri.dev.conf.json", import.meta.url),
);

export function withDevIdentity(args: string[]): string[] {
  const [command, ...rest] = args;
  // Flags after `--` belong to the app / cargo, not to the Tauri CLI.
  const end = rest.indexOf("--");
  const cliFlags = end === -1 ? rest : rest.slice(0, end);

  const isDevBuild =
    command === "dev" || (command === "build" && cliFlags.includes("--debug"));
  const hasConfig = cliFlags.some(
    (a) => a === "--config" || a === "-c" || a.startsWith("--config="),
  );

  if (!isDevBuild || hasConfig) return args;
  return [command, "--config", DEV_CONFIG, ...rest];
}

if (import.meta.main) {
  const proc = Bun.spawn(["tauri", ...withDevIdentity(process.argv.slice(2))], {
    stdio: ["inherit", "inherit", "inherit"],
  });
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.on(signal, () => proc.kill(signal));
  }
  process.exit(await proc.exited);
}
