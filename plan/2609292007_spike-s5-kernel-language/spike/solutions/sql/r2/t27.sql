WITH p AS (
  SELECT json_extract(args, '$.pin_id') AS pin_id,
         json_extract(args, '$.action') AS action,
         ROW_NUMBER() OVER (
           PARTITION BY json_extract(args, '$.pin_id')
           ORDER BY writer DESC, seq DESC
         ) AS rn
  FROM events
  WHERE kind = 'pin'
    AND json_extract(args, '$.action') IN ('add', 'remove')
)
SELECT pin_id
FROM p
WHERE rn = 1
  AND action = 'add'
  AND pin_id IS NOT NULL
ORDER BY pin_id ASC;
