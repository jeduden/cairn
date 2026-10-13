SELECT COUNT(*)
FROM events
WHERE kind = 'message'
  AND provenance = 'user'
  AND instr(lower(text), 'test') > 0;
