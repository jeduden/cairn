SELECT run, SUM(tokens) AS tokens
FROM events
WHERE kind = 'message'
  AND provenance = 'assistant'
  AND run IS NOT NULL
GROUP BY run
ORDER BY tokens DESC, run ASC
LIMIT 3;
