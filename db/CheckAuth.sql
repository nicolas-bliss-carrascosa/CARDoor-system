-- assuming that the user and door exist, and are 2, 1 resp
SELECT EXISTS (
    SELECT 1
    FROM FacilityXRoleXTimeRange    AS frtr
    INNER JOIN Doors                AS d  
        USING (FacilityId)
    INNER JOIN UserXRole            AS ur 
        USING (RoleId)
    LEFT JOIN DoorXRole
        USING (DoorId, RoleID)
    WHERE DoorId = ?1
        AND UserId = ?2
        AND Allowed IS NOT FALSE
        AND strftime('%H:%M:%S', 'now')
            BETWEEN StartTime AND EndTime

    UNION
    SELECT 1
    FROM DoorXRoleXTimeRange        AS drtr
    INNER JOIN DoorXRole            as dr
        USING (DoorId)
    INNER JOIN UserXRole            AS ur 
        USING (RoleId)
    WHERE DoorId = ?1
        AND UserId = ?2
        AND Allowed = 1
        AND strftime('%H:%M:%S', 'now')
            BETWEEN StartTime AND EndTime 
)