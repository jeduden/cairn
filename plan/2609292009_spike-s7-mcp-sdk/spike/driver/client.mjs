import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
for (const bin of process.argv.slice(2)) {
  const c = new Client({ name: "ts-check", version: "0" });
  await c.connect(new StdioClientTransport({ command: bin }));
  const tools = await c.listTools();
  const r = await c.callTool({ name: "event_search", arguments: { query: "pin" } });
  const ac = new AbortController();
  const slow = c.callTool({ name: "event_search", arguments: { query: "slow" } }, undefined, { signal: ac.signal }).then(() => "answered", e => "aborted: " + e.message.slice(0, 40));
  setTimeout(() => ac.abort(), 50);
  const s = await slow;
  await c.ping();
  console.log(bin.split("/").pop(), c.getServerVersion()?.name, JSON.stringify({ tools: tools.tools.map(t => t.name), structured: r.structuredContent, slow: s }));
  await c.close();
}
