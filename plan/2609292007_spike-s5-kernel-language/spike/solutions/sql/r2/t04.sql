SELECT COUNT(DISTINCT json_extract(args, '$.file_path'))
FROM events
WHERE kind = 'tool_call'
  AND tool IN ('Edit', 'Write')
  AND json_extract(args, '$.file_path') IS NOT NULL;
