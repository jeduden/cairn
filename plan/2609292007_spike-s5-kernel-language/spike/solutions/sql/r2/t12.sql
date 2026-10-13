SELECT c.tool AS tool, AVG(CAST(r.ts - c.ts AS REAL)) AS avg_seconds
FROM events c
JOIN events r ON r.kind = 'tool_result' AND r.call_id = c.call_id
WHERE c.kind = 'tool_call'
GROUP BY c.tool
ORDER BY c.tool ASC;
