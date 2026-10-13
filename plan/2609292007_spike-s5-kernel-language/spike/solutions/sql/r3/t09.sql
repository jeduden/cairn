SELECT run, SUM(CASE WHEN kind = 'compaction' THEN 1 ELSE 0 END) AS compactions
FROM events
WHERE run IS NOT NULL
GROUP BY run
ORDER BY run ASC;
