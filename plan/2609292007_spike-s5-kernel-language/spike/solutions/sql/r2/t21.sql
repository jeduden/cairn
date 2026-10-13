SELECT COALESCE(SUM(CAST(json_extract(args, '$.pre_tokens') AS INTEGER)), 0) AS total
FROM events
WHERE kind = 'compaction'
  AND json_extract(args, '$.trigger') = 'auto';
