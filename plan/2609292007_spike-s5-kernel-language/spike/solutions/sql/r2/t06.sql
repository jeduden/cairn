SELECT tool, COUNT(*) AS calls
FROM events
WHERE kind = 'tool_call'
GROUP BY tool
ORDER BY calls DESC, tool ASC;
