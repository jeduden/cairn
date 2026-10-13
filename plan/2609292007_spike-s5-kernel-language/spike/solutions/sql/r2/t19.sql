SELECT COUNT(DISTINCT CAST(floor(ts / 86400.0) AS INTEGER))
FROM events
WHERE kind = 'message' AND provenance = 'user';
