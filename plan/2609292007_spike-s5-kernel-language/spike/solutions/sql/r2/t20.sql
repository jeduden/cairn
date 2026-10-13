WITH g AS (
  SELECT writer, seq,
         ts - LAG(ts) OVER (PARTITION BY writer ORDER BY seq) AS gap
  FROM events
)
SELECT writer, seq, gap
FROM g
WHERE gap IS NOT NULL
ORDER BY gap DESC, writer ASC, seq ASC
LIMIT 1;
