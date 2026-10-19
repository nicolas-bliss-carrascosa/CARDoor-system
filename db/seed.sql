BEGIN;

-- ---------------------------------------------------------------- users app
INSERT INTO "Users" ("UserId", "Name", "DateIssued") VALUES
  (1, 'Alice Admin',    '2026-01-05 09:00:00'),
  (2, 'Bob Employee',   '2026-02-10 09:00:00'),
  (3, 'Carol Contract', '2026-03-01 09:00:00'),
  (4, 'Dan Custodian',  '2026-03-15 09:00:00'),
  (5, 'Eve Employee',   '2026-04-20 09:00:00'),
  (6, 'Frank NewHire',  '2026-10-19 09:00:00');  -- no roles yet

INSERT INTO "Roles" ("RoleId", "description") VALUES
  (1, 'Admin'),
  (2, 'Employee'),
  (3, 'Contractor'),
  (4, 'Janitorial');

INSERT INTO "UserXRole" ("UserId", "UserRoleId", "DateIssued") VALUES
  (1, 1, '2026-01-05 09:05:00'),
  (2, 2, '2026-02-10 09:05:00'),
  (3, 3, '2026-03-01 09:05:00'),
  (4, 4, '2026-03-15 09:05:00'),
  (5, 2, '2026-04-20 09:05:00');

-- ----------------------------------------------------------- facilities app
INSERT INTO "Facilities" ("FacilityId", "Name") VALUES
  (1, 'Headquarters'),
  (2, 'Warehouse');

-- ExpectedState: TRUE = expected locked
INSERT INTO "Doors" ("DoorId", "FacilityId", "Name", "ExpectedState") VALUES
  (1, 1, 'HQ Front Entrance', TRUE),
  (2, 1, 'HQ Server Room',    TRUE),
  (3, 1, 'HQ Lab',            TRUE),
  (4, 2, 'Warehouse Dock',    TRUE),
  (5, 2, 'Warehouse Office',  TRUE),
  (6, 2, 'Warehouse Cage',    TRUE);

-- Facility-wide access by role. NULL ValidFrom/ValidTo = unbounded.
INSERT INTO "FacilityXRole" ("FacilityId", "RoleId", "DateIssued", "ValidFrom", "ValidTo") VALUES
  (1, 1, '2026-01-05 09:10:00', NULL,       NULL),        -- Admin: HQ, always
  (2, 1, '2026-01-05 09:10:00', NULL,       NULL),        -- Admin: Warehouse, always
  (1, 2, '2026-02-10 09:10:00', '08:00:00', '18:00:00'),  -- Employee: HQ, business hours
  (2, 3, '2026-03-01 09:10:00', '07:00:00', '17:00:00'),  -- Contractor: Warehouse, daytime
  (1, 4, '2026-03-15 09:10:00', '18:00:00', '23:59:59'),  -- Janitorial: HQ, evenings
  (2, 4, '2026-03-15 09:10:00', '18:00:00', '23:59:59');  -- Janitorial: Warehouse, evenings

-- Exception layer: role -> single door
INSERT INTO "DoorXRole" ("RoleId", "DoorId", "Allowed", "DateIssued", "ValidFrom", "ValidTo") VALUES
  (2, 3, TRUE,  '2026-02-10 09:15:00', '09:00:00', '17:00:00'),  -- Employee may use the Lab
  (2, 2, FALSE, '2026-02-10 09:15:00', NULL,       NULL),        -- Employee denied Server Room
  (3, 6, FALSE, '2026-03-01 09:15:00', NULL,       NULL),        -- Contractor denied Warehouse Cage
  (4, 2, FALSE, '2026-03-15 09:15:00', NULL,       NULL);        -- Janitorial denied Server Room

-- Exception layer: user -> single door
INSERT INTO "UserXDoor" ("UserId", "DoorId", "Allowed", "DateIssued", "ValidFrom", "ValidTo") VALUES
  (2, 2, TRUE,  '2026-05-01 10:00:00', NULL,       NULL),        -- Bob granted Server Room
  (3, 1, TRUE,  '2026-03-01 09:20:00', '08:00:00', '17:00:00'),  -- Carol granted HQ front door
  (5, 3, FALSE, '2026-09-01 10:00:00', NULL,       NULL);        -- Eve revoked from Lab

-- ------------------------------------------------------------ dashboard app
-- Door access attempts (Result: TRUE = granted)
INSERT INTO "Logs" ("DoorId", "UserId", "Date", "Result") VALUES
  (1, 1, '2026-10-19 08:02:11', TRUE),
  (1, 2, '2026-10-19 08:15:40', TRUE),
  (1, 5, '2026-10-19 08:31:05', TRUE),
  (3, 2, '2026-10-19 09:10:22', TRUE),
  (3, 5, '2026-10-19 09:12:57', FALSE),   -- Eve revoked from Lab
  (2, 2, '2026-10-19 10:03:18', TRUE),    -- Bob's Server Room exception
  (2, 5, '2026-10-19 10:20:44', FALSE),
  (4, 3, '2026-10-19 07:05:30', TRUE),
  (6, 3, '2026-10-19 07:42:09', FALSE),   -- Contractor denied Cage
  (5, 1, '2026-10-19 11:00:00', TRUE),
  (1, 6, '2026-10-19 09:30:12', FALSE),   -- Frank has no roles
  (1, 4, '2026-10-18 18:45:03', TRUE),
  (2, 4, '2026-10-18 19:10:51', FALSE);   -- Janitorial denied Server Room

-- Door heartbeat messages
INSERT INTO "LivenessLog" ("DoorId", "Date", "Sent") VALUES
  (1, '2026-10-19 11:00:00', 'PING'),
  (1, '2026-10-19 11:05:00', 'PING'),
  (2, '2026-10-19 11:00:00', 'PING'),
  (3, '2026-10-19 11:00:00', 'PING'),
  (4, '2026-10-19 11:00:00', 'PING'),
  (5, '2026-10-19 11:00:00', 'PING'),
  (6, '2026-10-19 10:00:00', 'PING');   -- Cage has gone quiet since 10:00

COMMIT;