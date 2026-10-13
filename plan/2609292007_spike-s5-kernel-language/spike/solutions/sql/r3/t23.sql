SELECT q AS query
FROM (
  SELECT json_extract(args, '$.query') AS q
  FROM events
  WHERE kind = 'recall'
)
WHERE q IS NOT NULL
GROUP BY q
HAVING COUNT(*) > 1
ORDER BY q ASC;
