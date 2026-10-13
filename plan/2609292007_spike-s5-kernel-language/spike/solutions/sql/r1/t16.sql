SELECT COUNT(*)
FROM events e
WHERE e.kind = 'tool_call'
  AND e.tool = 'Edit'
  AND NOT EXISTS (
    SELECT 1 FROM events rd
    WHERE rd.kind = 'tool_call'
      AND rd.tool = 'Read'
      AND rd.writer = e.writer
      AND rd.seq < e.seq
      AND json_extract(rd.args, '$.file_path') = json_extract(e.args, '$.file_path')
  );
