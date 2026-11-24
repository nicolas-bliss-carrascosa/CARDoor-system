-- by default, all timestamps are seconds since unix-epoch
-- NOTE: foreign keys are only enforced if you run `PRAGMA foreign_keys = ON;`
-- on every connection (SQLite default is OFF).

-- the times in timeRanges are HH:MM:SS format, UTC
-- timeRanges don't support wrapping around midnight (though 24:00:00 is supported as a midnight)

-- bools are just integers with 0 as false and 1 as true

CREATE TABLE IF NOT EXISTS `Users` (
  `UserId` integer PRIMARY KEY,
  `Name` varchar(255),
  `DateIssued` timestamp
);
 
CREATE TABLE IF NOT EXISTS `Roles` (
  `RoleId` integer PRIMARY KEY,
  `description` varchar(255)
);

CREATE TABLE IF NOT EXISTS `Facilities` (
  `FacilityId` integer PRIMARY KEY,
  `Name` varchar(255)
);
 
CREATE TABLE IF NOT EXISTS `Doors` (
  `DoorId` integer PRIMARY KEY,
  `FacilityId` integer NOT NULL REFERENCES `Facilities` (`FacilityId`),
  `Name` varchar(255),
  `ExpectedState` bool -- standin for more complicated enum !
);
 
CREATE TABLE IF NOT EXISTS `UserXRole` (
  `UserId` integer REFERENCES `Users` (`UserId`),
  `RoleId` integer REFERENCES `Roles` (`RoleId`),
  `DateIssued` timestamp,
  PRIMARY KEY (`UserId`, `RoleId`)
);
 
CREATE TABLE IF NOT EXISTS `FacilityXRole` (
  `FacilityId` integer REFERENCES `Facilities` (`FacilityId`),
  `RoleId` integer REFERENCES `Roles` (`RoleId`),
  `DateIssued` timestamp,
  PRIMARY KEY (`FacilityId`, `RoleId`)
);

-- exception layer: let a role access one particular door
CREATE TABLE IF NOT EXISTS `DoorXRole` (
  `RoleId` integer REFERENCES `Roles` (`RoleId`),
  `DoorId` integer REFERENCES `Doors` (`DoorId`),
  `Allowed` bool, -- here to allow doors to be cut out of facility-wide rules (does not generically block someone with the role)
  `DateIssued` timestamp,
  PRIMARY KEY (`RoleId`, `DoorId`)
);

-- the role has access to this facility if the current time is in _any_ of the ranges!
CREATE TABLE IF NOT EXISTS `FacilityXRoleXTimeRange` (
	`RoleId` integer REFERENCES `Roles` (`RoleId`),
	`FacilityId` integer REFERENCES `Facilities` (`FacilityId`),
	`StartTime` text NOT NULL
    CHECK (typeof(StartTime) = 'text' AND time(StartTime) IS StartTime),
	`EndTime` text NOT NULL
    CHECK (typeof(EndTime) = 'text' AND time(EndTime) IS EndTime),
	PRIMARY KEY (`RoleId`, `FacilityId`, `StartTime`, `EndTime`)
);

CREATE TABLE IF NOT EXISTS `DoorXRoleXTimeRange` (
	`RoleId` integer REFERENCES `Roles` (`RoleId`),
	`DoorId` integer REFERENCES `Doors` (`DoorId`),
	`StartTime` text NOT NULL
    CHECK (typeof(StartTime) = 'text' AND time(StartTime) IS StartTime),
	`EndTime` text NOT NULL
    CHECK (typeof(EndTime) = 'text' AND time(EndTime) IS EndTime),
	PRIMARY KEY (`RoleId`, `DoorId`, `StartTime`, `EndTime`)
);

-- this is appendonly
CREATE TABLE IF NOT EXISTS `Logs` (
  `DoorId` integer REFERENCES `Doors` (`DoorId`),
  `UserId` integer REFERENCES `Users` (`UserId`),
  `Date` timestamp,
  `Result` bool, 
  PRIMARY KEY (`DoorId`, `UserId`, `Date`)
);
 
-- this is appendonly
CREATE TABLE IF NOT EXISTS `LivenessLog` (
  `DoorId` integer REFERENCES `Doors` (`DoorId`),
  `Date` timestamp,
  `Sent` str,
  PRIMARY KEY (`DoorId`, `Date`)
);
 
-- you can't do the alter table stuff in sqlite
