SELECT COUNT(*)
FROM events e
WHERE e.kind = 'tool_call'
  AND e.tool = 'Edit'
  AND NOT EXISTS (
    SELECT 1 FROM events r
    WHERE r.writer = e.writer
      AND r.kind = 'tool_call'
      AND r.tool = 'Read'
      AND r.seq < e.seq
      AND json_extract(r.args, '$.file_path') = json_extract(e.args, '$.file_path')
  );
