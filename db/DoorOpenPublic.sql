-- assuming that the door exists, 
SELECT EXISTS (
    SELECT 1
    FROM FacilityXRole
    INNER JOIN Doors
        USING (FacilityId)
    INNER JOIN FacilityXRoleXTimeRange
        USING (FacilityId, RoleId)
    WHERE DoorId = ?1
        AND RoleId = 1 -- remember that 1 is hardcoded to be the public role!!
        AND strftime('%H:%M:%S', 'now')
            BETWEEN StartTime AND EndTime
    
    UNION
    SELECT 1
    FROM DoorXRole
    INNER JOIN DoorXRoleXTimeRange
        USING (DoorId, RoleId)
    WHERE DoorId = ?1
        AND RoleId = 1 -- remember that 1 is hardcoded to be the public role!!
        AND strftime('%H:%M:%S', 'now')
            BETWEEN StartTime AND EndTime 
)