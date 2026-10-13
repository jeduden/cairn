WITH calls AS (
  SELECT writer, seq, tool, args, call_id,
         LAG(tool) OVER (PARTITION BY writer ORDER BY seq) AS prev_tool,
         LAG(args) OVER (PARTITION BY writer ORDER BY seq) AS prev_args,
         LAG(call_id) OVER (PARTITION BY writer ORDER BY seq) AS prev_call_id,
         ROW_NUMBER() OVER (PARTITION BY writer ORDER BY seq) AS rn
  FROM events
  WHERE kind = 'tool_call'
)
SELECT COUNT(*) AS retries
FROM calls c
WHERE c.rn > 1
  AND c.tool = c.prev_tool
  AND c.args = c.prev_args
  AND EXISTS (
    SELECT 1
    FROM events r
    WHERE r.kind = 'tool_result'
      AND r.call_id = c.prev_call_id
      AND r.status = 'error'
  );
