WITH calls AS (
  SELECT
    json_extract(c.args, '$.command') AS command,
    EXISTS (
      SELECT 1 FROM events r
      WHERE r.kind = 'tool_result'
        AND r.call_id = c.call_id
        AND r.status = 'error'
    ) AS failed
  FROM events c
  WHERE c.kind = 'tool_call'
    AND c.tool = 'Bash'
)
SELECT command, COUNT(*) AS runs, SUM(failed) AS failures
FROM calls
GROUP BY command
HAVING SUM(failed) > 0
ORDER BY failures DESC, command ASC;
