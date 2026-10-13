SELECT c.call_id
FROM events c
JOIN events r
  ON r.kind = 'tool_result'
 AND r.call_id = c.call_id
WHERE c.kind = 'tool_call'
  AND c.call_id IS NOT NULL
ORDER BY (r.ts - c.ts) DESC, c.call_id ASC
LIMIT 1;
