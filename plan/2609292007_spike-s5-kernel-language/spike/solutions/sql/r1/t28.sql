WITH numbered AS (
  SELECT writer, seq, run, kind, provenance, status,
         SUM(CASE WHEN kind = 'message' AND provenance = 'user' THEN 1 ELSE 0 END)
           OVER (PARTITION BY writer ORDER BY seq ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS turn
  FROM events
),
turn_starts AS (
  SELECT writer, turn, run
  FROM numbered
  WHERE kind = 'message' AND provenance = 'user'
),
error_turns AS (
  SELECT DISTINCT writer, turn
  FROM numbered
  WHERE turn > 0
    AND kind = 'tool_result'
    AND status = 'error'
)
SELECT s.run AS run, COUNT(*) AS turns
FROM error_turns e
JOIN turn_starts s ON s.writer = e.writer AND s.turn = e.turn
WHERE s.run IS NOT NULL
GROUP BY s.run
ORDER BY s.run ASC;
