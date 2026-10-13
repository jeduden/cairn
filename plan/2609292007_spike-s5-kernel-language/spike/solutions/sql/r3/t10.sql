SELECT json_extract(args, '$.file_path') AS file_path, COUNT(*) AS reads
FROM events
WHERE kind = 'tool_call' AND tool = 'Read'
  AND json_extract(args, '$.file_path') IS NOT NULL
GROUP BY file_path
ORDER BY reads DESC, file_path ASC
LIMIT 5;
