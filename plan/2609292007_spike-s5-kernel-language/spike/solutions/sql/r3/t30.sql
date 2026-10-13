WITH e AS (
  SELECT run, seq,
         SUM(tokens) OVER (PARTITION BY run ORDER BY seq ROWS UNBOUNDED PRECEDING) AS cs
  FROM events
  WHERE run IS NOT NULL
)
SELECT run, MIN(seq) AS seq
FROM e
WHERE cs >= 50000
GROUP BY run
ORDER BY run ASC;
