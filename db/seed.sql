-- I DID USE AI FOR THIS PART but if you want to handmake data go ahead I'm impressed you have all that patience



-- Safe to re-run: it clears the tables first (children before parents).
-- Enable FK checking on your connection with: PRAGMA foreign_keys = ON;
--
-- Conventions:
--   timestamps  -> 'YYYY-MM-DD HH:MM:SS' text
--   ValidRange  -> 'start/end' text (ISO dates), or NULL for "no restriction"
--   bool        -> 1 / 0


BEGIN TRANSACTION;

DELETE FROM `LivenessLog`;
DELETE FROM `Logs`;
DELETE FROM `UserXDoor`;
DELETE FROM `DoorXRole`;
DELETE FROM `FacilityXRole`;
DELETE FROM `UserXRole`;
DELETE FROM `Doors`;
DELETE FROM `Facilities`;
DELETE FROM `Roles`;
DELETE FROM `Users`;

-- ---------------------------------------------------------------- Users
INSERT INTO `Users` (`UserId`, `Name`, `DateIssued`) VALUES
  (1,  'Helen Okafor',    '2022-07-01 09:00:00'),   -- registrar
  (2,  'Walter Greene',   '2023-01-09 08:30:00'),   -- facilities manager
  (3,  'Marcus Lindgren', '2023-08-14 10:00:00'),   -- professor
  (4,  'Priya Raman',     '2024-08-12 10:15:00'),   -- professor
  (5,  'Tomás Alvarez',   '2025-01-13 09:45:00'),   -- professor
  (6,  'Omar Haddad',     '2026-08-24 11:00:00'),   -- grad student + teaching assistant
  (7,  'Sofia Rossi',     '2024-08-26 12:00:00'),
  (8,  'Ethan Walker',    '2025-08-25 12:10:00'),
  (9,  'Aaliyah Brooks',  '2026-08-24 12:20:00'),
  (10, 'Noah Kim',        '2026-08-24 12:25:00'),
  (11, 'Maya Patel',      '2023-08-28 12:30:00'),   -- senior doing thesis research
  (12, 'Liam O''Connor',  '2026-09-15 14:05:00');   -- transfer student

-- ---------------------------------------------------------------- Roles
INSERT INTO `Roles` (`RoleId`, `description`) VALUES
  (1, 'Administrator'),
  (2, 'Faculty'),
  (3, 'Student');

-- ----------------------------------------------------------- Facilities
INSERT INTO `Facilities` (`FacilityId`, `Name`) VALUES
  (1, 'Main Hall'),
  (2, 'Science Building'),
  (3, 'Library'),
  (4, 'Athletic Center');

-- ---------------------------------------------------------------- Doors
-- Name is optional (nullable); door 10 is left unnamed on purpose.
INSERT INTO `Doors` (`DoorId`, `FacilityId`, `Name`, `ExpectedState`) VALUES
  (1,  1, 'Front Entrance',      1),   -- Main Hall
  (2,  1, 'Registrar''s Office', 1),
  (3,  1, 'Classroom 101',       0),
  (4,  2, 'Main Entrance',       1),   -- Science Building
  (5,  2, 'Chemistry Lab',       1),
  (6,  2, 'Physics Lab',         1),
  (7,  3, 'Main Entrance',       0),   -- Library
  (8,  3, 'Special Collections', 1),
  (9,  4, 'Gym Entrance',        0),   -- Athletic Center
  (10, 4, NULL,                  1);   -- (equipment room, unnamed)
 


-- ----------------------------------------------------------- UserXRole
INSERT INTO `UserXRole` (`UserId`, `UserRoleId`, `DateIssued`) VALUES
  (1,  1, '2022-07-01 09:00:00'),
  (2,  1, '2023-01-09 08:30:00'),
  (3,  2, '2023-08-14 10:00:00'),
  (4,  2, '2024-08-12 10:15:00'),
  (5,  2, '2025-01-13 09:45:00'),
  (6,  3, '2026-08-24 11:00:00'),
  (6,  2, '2026-09-01 09:00:00'),
  (7,  3, '2024-08-26 12:00:00'),
  (8,  3, '2025-08-25 12:10:00'),
  (9,  3, '2026-08-24 12:20:00'),
  (10, 3, '2026-08-24 12:25:00'),
  (11, 3, '2023-08-28 12:30:00'),
  (12, 3, '2026-09-15 14:05:00');

-- ------------------------------------------------------- FacilityXRole
-- Which roles can get into which building, and when that grant is valid.
INSERT INTO `FacilityXRole` (`FacilityId`, `RoleId`, `DateIssued`, `ValidRange`) VALUES
  (1, 1, '2022-07-01 00:00:00', NULL),
  (1, 2, '2023-01-01 00:00:00', NULL),
  (1, 3, '2026-08-17 00:00:00', '2026-08-24/2026-12-18'),
  (2, 1, '2022-07-01 00:00:00', NULL),
  (2, 2, '2023-01-01 00:00:00', NULL),
  (2, 3, '2026-01-05 00:00:00', '2026-01-12/2026-05-15'),   -- spring term, expired
  (3, 1, '2022-07-01 00:00:00', NULL),
  (3, 2, '2023-01-01 00:00:00', NULL),
  (3, 3, '2023-01-01 00:00:00', NULL),
  (4, 1, '2022-07-01 00:00:00', NULL),
  (4, 2, '2023-01-01 00:00:00', NULL),
  (4, 3, '2026-08-17 00:00:00', '2026-08-24/2026-12-18');

-- ------------------------------------------------------------ DoorXRole
-- Exceptions: a role gets (or loses) access to one specific door.
INSERT INTO `DoorXRole` (`RoleId`, `DoorId`, `Allowed`, `DateIssued`, `ValidRange`) VALUES
  (2, 2, 0, '2023-01-01 00:00:00', NULL),                     -- Faculty: no registrar's office
  (3, 2, 0, '2023-01-01 00:00:00', NULL),                     -- Students: no registrar's office
  (3, 4, 1, '2026-08-17 00:00:00', '2026-08-24/2026-12-18'),  -- Students: science entrance this term
  (3, 8, 0, '2023-01-01 00:00:00', NULL),                     -- Students: no archives by default
  (3, 10, 0, '2023-01-01 00:00:00', NULL);                    -- Students: no equipment room

-- ------------------------------------------------------------ UserXDoor
-- Exceptions: one user gets (or loses) access to one specific door.
INSERT INTO `UserXDoor` (`UserId`, `DoorId`, `Allowed`, `DateIssued`, `ValidRange`) VALUES
  (7,  5, 1, '2026-08-24 09:00:00', '2026-08-24/2026-12-18'),  -- Sofia: enrolled in chem lab
  (8,  5, 1, '2026-08-24 09:00:00', '2026-08-24/2026-12-18'),  -- Ethan: enrolled in chem lab
  (10, 5, 0, '2026-10-12 15:00:00', NULL),                     -- Noah: lab safety hold
  (11, 8, 1, '2026-10-01 10:00:00', '2026-10-01/2026-11-30');  -- Maya: thesis research in archives

-- ----------------------------------------------------------------- Logs
INSERT INTO `Logs` (`DoorId`, `UserId`, `Date`, `Result`) VALUES
  -- Thursday 10/15
  (1,  1,  '2026-10-15 07:45:12', 1),
  (2,  1,  '2026-10-15 07:50:30', 1),
  (1,  3,  '2026-10-15 08:05:44', 1),
  (3,  3,  '2026-10-15 08:10:02', 1),
  (1,  7,  '2026-10-15 08:40:19', 1),
  (3,  7,  '2026-10-15 08:52:41', 1),
  (2,  3,  '2026-10-15 09:30:08', 0),
  (4,  8,  '2026-10-15 10:55:23', 1),
  (5,  8,  '2026-10-15 11:02:10', 1),
  (5,  10, '2026-10-15 11:04:47', 0),
  (5,  4,  '2026-10-15 11:20:36', 1),
  (7,  11, '2026-10-15 13:15:00', 1),
  (8,  11, '2026-10-15 13:40:52', 1),
  (8,  9,  '2026-10-15 14:05:33', 0),
  (9,  12, '2026-10-15 16:30:28', 1),
  -- Friday 10/16
  (1,  2,  '2026-10-16 07:30:15', 1),
  (2,  2,  '2026-10-16 07:42:09', 1),
  (1,  6,  '2026-10-16 08:25:50', 1),
  (3,  6,  '2026-10-16 08:31:12', 1),
  (6,  5,  '2026-10-16 09:10:44', 1),
  (6,  9,  '2026-10-16 09:20:05', 0),
  (4,  7,  '2026-10-16 10:00:18', 1),
  (5,  7,  '2026-10-16 10:05:29', 1),
  (7,  9,  '2026-10-16 15:12:36', 1),
  (9,  10, '2026-10-16 17:45:01', 1),
  -- Saturday 10/17
  (7,  8,  '2026-10-17 10:20:14', 1),
  (7,  12, '2026-10-17 11:05:48', 1),
  (8,  11, '2026-10-17 11:50:09', 1),
  (9,  10, '2026-10-17 14:30:40', 1),
  (10, 12, '2026-10-17 14:42:57', 0),
  -- Sunday 10/18
  (7,  7,  '2026-10-18 13:00:22', 1),
  (2,  9,  '2026-10-18 13:20:00', 0),
  (7,  4,  '2026-10-18 14:10:11', 1);

-- ---------------------------------------------------------- LivenessLog
-- Door 6 (physics lab) was last heard from on the 16th, so it looks offline.
INSERT INTO `LivenessLog` (`DoorId`, `Date`, `Sent`) VALUES
  (1,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (1,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (2,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (2,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (3,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (3,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (4,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (4,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (5,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (5,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (6,  '2026-10-16 03:10:00', 'HEARTBEAT'),
  (7,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (7,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (8,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (9,  '2026-10-18 22:00:00', 'HEARTBEAT'),
  (9,  '2026-10-18 22:15:00', 'HEARTBEAT'),
  (10, '2026-10-18 22:00:00', 'HEARTBEAT'),
  (10, '2026-10-18 22:15:00', 'HEARTBEAT');

COMMIT;