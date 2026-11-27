BEGIN;
WITH RECURSIVE
    door_ids(id) AS (
        SELECT 1
        UNION ALL
        SELECT id + 1 FROM door_ids WHERE id < 6
    ),
    hour_offsets(n) AS (
        SELECT 0
        UNION ALL
        SELECT n + 1 FROM hour_offsets WHERE n < (14 * 24) - 1
    )
INSERT OR IGNORE INTO `LivenessLog` (`DoorId`, `Date`, `Sent`)
SELECT
    d.id,
    strftime('%Y-%m-%d %H:00:00', 'now', 'localtime', '-' || h.n || ' hours'),
    'PING'
FROM door_ids AS d
CROSS JOIN hour_offsets AS h;
COMMIT;