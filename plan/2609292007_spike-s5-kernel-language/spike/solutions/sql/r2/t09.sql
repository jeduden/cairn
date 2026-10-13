SELECT r.run,
       (SELECT COUNT(*) FROM events e WHERE e.run = r.run AND e.kind = 'compaction') AS compactions
FROM (SELECT DISTINCT run FROM events WHERE run IS NOT NULL) AS r
ORDER BY r.run ASC;
