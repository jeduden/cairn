WITH results AS (
  SELECT writer, seq, status,
         SUM(CASE WHEN status = 'error' THEN 0 ELSE 1 END)
           OVER (PARTITION BY writer ORDER BY seq ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS grp
  FROM events
  WHERE kind = 'tool_result'
),
streaks AS (
  SELECT writer, grp, COUNT(*) AS len
  FROM results
  WHERE status = 'error'
  GROUP BY writer, grp
)
SELECT COALESCE(MAX(len), 0) AS longest
FROM streaks;
