-- assuming that the door exists, 
SELECT EXISTS (
    SELECT 1
    FROM FacilityXRole as fr
    INNER JOIN Doors AS d
        USING (FacilityId)
    WHERE d.DoorId = $1
        AND fr.RoleId = 1 -- remember that 1 is hardcoded to be the public role!!
        AND strftime('%H:%M:%S', 'now')
            BETWEEN fr.ValidRangeStart AND fr.ValidRangeEnd
)