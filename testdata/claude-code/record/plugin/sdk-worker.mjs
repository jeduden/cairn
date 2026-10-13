// sdk-worker.mjs SDK_DIR PLUGIN_DIR PROMPT — spike S4: an Agent SDK worker that
// loads the stub Cairn plugin by path and prints every SDK message as one JSON
// line. Run it through ./run with REC_BIN=node, so the worker and the harness
// it starts see a clean environment.
import { pathToFileURL } from "node:url";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const [sdkDir, pluginDir, prompt] = process.argv.slice(2);
const sdk = await import(
  pathToFileURL(join(sdkDir, "node_modules/@anthropic-ai/claude-agent-sdk/sdk.mjs")).href
);
const claude = execFileSync("sh", ["-c", "command -v claude"]).toString().trim();
for await (const message of sdk.query({
  prompt,
  options: {
    plugins: [{ type: "local", path: pluginDir }],
    pathToClaudeCodeExecutable: claude,
    settingSources: [],
    permissionMode: "default",
    allowedTools: ["mcp__plugin_cairn_cairn__event_search", "Agent"],
    model: "sonnet",
  },
})) {
  process.stdout.write(JSON.stringify(message) + "\n");
}
