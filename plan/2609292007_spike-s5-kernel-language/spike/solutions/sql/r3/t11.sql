SELECT writer, COUNT(*) AS events, MAX(seq) AS max_seq
FROM events
GROUP BY writer
ORDER BY writer ASC;
