SELECT CAST(strftime('%H', ts, 'unixepoch') AS INTEGER) AS hour, COUNT(*) AS messages
FROM events
WHERE kind = 'message' AND provenance = 'user'
GROUP BY hour
ORDER BY hour ASC;
