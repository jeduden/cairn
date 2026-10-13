SELECT json_extract(args, '$.query') AS query
FROM events
WHERE kind = 'recall'
  AND json_extract(args, '$.query') IS NOT NULL
GROUP BY json_extract(args, '$.query')
HAVING COUNT(*) > 1
ORDER BY query ASC;
