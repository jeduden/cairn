WITH t AS (
  SELECT writer, run, seq, kind, status,
         SUM(CASE WHEN kind = 'message' AND provenance = 'user' THEN 1 ELSE 0 END)
           OVER (PARTITION BY writer ORDER BY seq ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS turn
  FROM events
)
SELECT run, COUNT(DISTINCT writer || ':' || turn) AS turns
FROM t
WHERE turn > 0
  AND kind = 'tool_result'
  AND status = 'error'
  AND run IS NOT NULL
GROUP BY run
ORDER BY run ASC;
