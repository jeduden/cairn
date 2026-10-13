SELECT run, MAX(ts) - MIN(ts) AS span
FROM events
WHERE run IS NOT NULL
GROUP BY run
ORDER BY span DESC, run ASC;
