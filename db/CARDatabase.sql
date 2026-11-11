-- by default, all timestamps are seconds since unix-epoch
-- NOTE: foreign keys are only enforced if you run `PRAGMA foreign_keys = ON;`
-- on every connection (SQLite default is OFF).

CREATE TABLE `Users` (
  `UserId` integer PRIMARY KEY,
  `DateIssued` timestamp
);

CREATE TABLE `Roles` (
  `RoleId` integer PRIMARY KEY,
  `Description` varchar(255)
);

CREATE TABLE `Facilities` (
  `FacilityId` integer PRIMARY KEY,
  `Name` varchar(255)
);

CREATE TABLE `UserXRole` (
  `UserId` integer NOT NULL REFERENCES `Users` (`UserId`),
  `RoleId` integer NOT NULL REFERENCES `Roles` (`RoleId`),
  `DateIssued` timestamp,
  PRIMARY KEY (`UserId`, `RoleId`)
);

CREATE TABLE `Doors` (
  `DoorId` integer PRIMARY KEY,
  `FacilityId` integer NOT NULL REFERENCES `Facilities` (`FacilityId`)
);

CREATE TABLE `FacilityXRole` (
  `RoleId` integer NOT NULL REFERENCES `Roles` (`RoleId`),
  `FacilityId` integer NOT NULL REFERENCES `Facilities` (`FacilityId`),
  `DateIssued` timestamp,
  -- TODO: if you mess this up the computer kills you
  -- here, the timestamps are HH:MM:SS format!
  `ValidRangeStart` text NOT NULL,
  `ValidRangeEnd` text NOT NULL,
  PRIMARY KEY (`FacilityId`, `RoleId`)
);

-- this is appendonly
CREATE TABLE `Logs` (
  `DoorId` integer NOT NULL REFERENCES `Doors` (`DoorId`),
  `UserId` integer NOT NULL REFERENCES `Users` (`UserId`),
  `Date` timestamp NOT NULL,
  `Result` bool,
  PRIMARY KEY (`DoorId`, `UserId`, `Date`)
);

-- this is appendonly
CREATE TABLE `LivenessLog` (
  `DoorId` integer NOT NULL REFERENCES `Doors` (`DoorId`),
  `Date` timestamp NOT NULL,
  `Sent` text,
  PRIMARY KEY (`DoorId`, `Date`)
);