SELECT COUNT(*)
FROM events c
WHERE c.kind = 'tool_call'
  AND NOT EXISTS (
    SELECT 1 FROM events r
    WHERE r.kind = 'tool_result' AND r.call_id = c.call_id
  );
