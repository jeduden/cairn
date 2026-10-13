WITH e AS (
  SELECT regexp_capture(json_extract(args, '$.file_path'), '\.([^.]*)$', 1) AS ext
  FROM events
  WHERE kind = 'tool_call'
    AND tool = 'Edit'
)
SELECT ext, COUNT(*) AS edits
FROM e
WHERE ext IS NOT NULL
GROUP BY ext
ORDER BY edits DESC, ext ASC;
