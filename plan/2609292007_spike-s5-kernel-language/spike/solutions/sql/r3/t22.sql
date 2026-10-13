SELECT code, COUNT(*) AS results
FROM (
  SELECT CAST(regexp_capture(text, 'exit code (-?\d+)', 1) AS INTEGER) AS code
  FROM events
  WHERE kind = 'tool_result'
    AND tool = 'Bash'
    AND status = 'error'
    AND regexp_capture(text, 'exit code (-?\d+)', 1) IS NOT NULL
)
GROUP BY code
ORDER BY code ASC;
