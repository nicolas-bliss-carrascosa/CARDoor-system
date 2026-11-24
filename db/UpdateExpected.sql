-- as always, assuming the door exists
UPDATE Doors
SET ExpectedState = ?2
WHERE DoorId = ?1