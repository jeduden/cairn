SELECT tool,
       SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) AS errors,
       COUNT(*) AS results,
       CAST(SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) AS REAL) / COUNT(*) AS rate
FROM events
WHERE kind = 'tool_result'
GROUP BY tool
ORDER BY rate DESC, tool ASC;
