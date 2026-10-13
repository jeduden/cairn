WITH codes AS (
  SELECT CAST(regexp_capture(text, 'exit code:?\s*(-?\d+)', 1) AS INTEGER) AS code
  FROM events
  WHERE kind = 'tool_result'
    AND tool = 'Bash'
    AND status = 'error'
)
SELECT code, COUNT(*) AS results
FROM codes
WHERE code IS NOT NULL
GROUP BY code
ORDER BY code ASC;
