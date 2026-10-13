WITH running AS (
  SELECT run, seq,
         SUM(tokens) OVER (PARTITION BY run ORDER BY seq ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS cum
  FROM events
  WHERE run IS NOT NULL
)
SELECT run, MIN(seq) AS seq
FROM running
WHERE cum >= 50000
GROUP BY run
ORDER BY run ASC;
