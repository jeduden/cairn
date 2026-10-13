SELECT COALESCE(SUM(json_extract(args, '$.pre_tokens')), 0) AS pre_tokens
FROM events
WHERE kind = 'compaction'
  AND json_extract(args, '$.trigger') = 'auto';
