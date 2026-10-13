WITH c AS (
  SELECT writer, seq, tool, args, call_id,
         LAG(tool) OVER w AS prev_tool,
         LAG(args) OVER w AS prev_args,
         LAG(call_id) OVER w AS prev_call_id
  FROM events
  WHERE kind = 'tool_call'
  WINDOW w AS (PARTITION BY writer ORDER BY seq)
)
SELECT COUNT(*) AS retries
FROM c
WHERE prev_call_id IS NOT NULL
  AND tool = prev_tool
  AND args = prev_args
  AND EXISTS (
    SELECT 1
    FROM events r
    WHERE r.kind = 'tool_result'
      AND r.call_id = c.prev_call_id
      AND r.writer = c.writer
      AND r.status = 'error'
  );
