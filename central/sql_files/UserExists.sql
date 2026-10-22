SELECT EXISTS (
    SELECT 1
    FROM Users
    WHERE UserId = ?1
)