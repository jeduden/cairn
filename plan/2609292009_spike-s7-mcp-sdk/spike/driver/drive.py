"""Drive an MCP stdio server with raw JSON-RPC and report what it did."""
import json, subprocess, sys, time, select

def run(binary):
    t0 = time.perf_counter()
    p = subprocess.Popen([binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
    def send(m): p.stdin.write(json.dumps(m) + "\n"); p.stdin.flush()
    def recv(timeout=2.0):
        r, _, _ = select.select([p.stdout], [], [], timeout)
        return json.loads(p.stdout.readline()) if r else None
    out = {}
    send({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"drive","version":"0"}}})
    init = recv(); out["init_ms"] = round((time.perf_counter()-t0)*1000, 2)
    out["protocolVersion"] = init["result"]["protocolVersion"]
    send({"jsonrpc":"2.0","method":"notifications/initialized"})
    send({"jsonrpc":"2.0","id":2,"method":"tools/list"})
    out["tools"] = [t["name"] for t in recv()["result"]["tools"]]
    lat = []
    for i in range(200):
        t = time.perf_counter()
        send({"jsonrpc":"2.0","id":100+i,"method":"tools/call","params":{"name":"event_search","arguments":{"query":"pin"}}})
        r = recv(); lat.append((time.perf_counter()-t)*1000)
    lat.sort(); out["call_p50_ms"] = round(lat[100],3); out["call_p95_ms"] = round(lat[190],3)
    out["structuredContent"] = r["result"].get("structuredContent")
    send({"jsonrpc":"2.0","id":"slow-1","method":"tools/call","params":{"name":"event_search","arguments":{"query":"slow"}}})
    time.sleep(0.05)
    send({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"slow-1","reason":"test"}})
    send({"jsonrpc":"2.0","id":3,"method":"ping"})
    seen = []
    while (m := recv(1.0)) is not None: seen.append(m.get("id"))
    out["after_cancel_ids"] = seen
    out["cancelled_got_no_response"] = "slow-1" not in seen and 3 in seen
    send({"jsonrpc":"2.0","id":4,"method":"nope"})
    out["unknown_method"] = (recv() or {}).get("error", {}).get("code")
    p.stdin.close(); p.wait(timeout=5); out["exit"] = p.returncode
    return out

for b in sys.argv[1:]:
    print(b.split("/")[-1], json.dumps(run(b), indent=1))
