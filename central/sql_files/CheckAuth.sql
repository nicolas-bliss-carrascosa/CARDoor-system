-- assuming that the user and door exist, and are 2, 1 resp
SELECT EXISTS (
    SELECT 1
    FROM FacilityXRole   AS fr
    INNER JOIN Doors     AS d  
        USING (FacilityId)
    INNER JOIN UserXRole AS ur 
        USING (RoleId)
    WHERE d.DoorId = $1
        AND ur.UserId = $2
        AND strftime('%H:%M:%S', 'now')
            BETWEEN fr.ValidRangeStart AND fr.ValidRangeEnd
)