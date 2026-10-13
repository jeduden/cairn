WITH e AS (
  SELECT run, seq, ts,
         LAG(ts) OVER (PARTITION BY run ORDER BY seq) AS prev_ts
  FROM events
  WHERE run IS NOT NULL
)
SELECT COUNT(*) AS sessions
FROM e
WHERE prev_ts IS NULL OR ts - prev_ts > 1800;
