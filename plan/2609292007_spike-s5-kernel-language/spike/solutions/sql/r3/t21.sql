SELECT COALESCE(SUM(json_extract(args, '$.pre_tokens')), 0) AS total
FROM events
WHERE kind = 'compaction'
  AND json_extract(args, '$.trigger') = 'auto';
