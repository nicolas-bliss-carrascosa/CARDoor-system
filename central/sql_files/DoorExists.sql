SELECT EXISTS (
    SELECT 1
    FROM Doors
    WHERE DoorId = ?1
)