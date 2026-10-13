WITH tr AS (
  SELECT writer, seq, status,
         SUM(CASE WHEN status = 'ok' THEN 1 ELSE 0 END)
           OVER (PARTITION BY writer ORDER BY seq ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS g
  FROM events
  WHERE kind = 'tool_result'
),
streaks AS (
  SELECT writer, g, COUNT(*) AS c
  FROM tr
  WHERE status = 'error'
  GROUP BY writer, g
)
SELECT COALESCE(MAX(c), 0) AS longest
FROM streaks;
