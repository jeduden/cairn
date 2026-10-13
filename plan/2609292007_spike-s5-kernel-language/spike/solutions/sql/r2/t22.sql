WITH r AS (
  SELECT CAST(regexp_capture(text, 'exit code\s*(-?\d+)', 1) AS INTEGER) AS code
  FROM events
  WHERE kind = 'tool_result'
    AND tool = 'Bash'
    AND status = 'error'
    AND regexp_capture(text, 'exit code\s*(-?\d+)', 1) IS NOT NULL
)
SELECT code, COUNT(*) AS results
FROM r
GROUP BY code
ORDER BY code ASC;
