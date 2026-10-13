WITH c AS (
  SELECT writer, seq, tool, args,
         LAG(tool) OVER w AS pt,
         LAG(args) OVER w AS pa,
         LAG(call_id) OVER w AS pc
  FROM events
  WHERE kind = 'tool_call'
  WINDOW w AS (PARTITION BY writer ORDER BY seq)
)
SELECT COUNT(*) AS retries
FROM c
WHERE c.pt = c.tool
  AND c.pa = c.args
  AND EXISTS (
    SELECT 1 FROM events r
    WHERE r.kind = 'tool_result'
      AND r.call_id = c.pc
      AND r.status = 'error'
  );
