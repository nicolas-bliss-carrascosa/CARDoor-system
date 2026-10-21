-- assuming that the user and door exist, and are $2, $1 resp
SELECT fr.ValidRange
FROM FacilityXRole   AS fr
INNER JOIN Doors     AS d  
    USING (FacilityId)
INNER JOIN UserXRole AS ur 
    USING (RoleId)
WHERE d.DoorId = $1
    AND ur.UserId = $2