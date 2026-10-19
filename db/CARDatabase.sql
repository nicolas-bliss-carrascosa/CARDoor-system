
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
  `UserRoleId` integer REFERENCES `Roles` (`RoleId`),
  `DateIssued` timestamp,
  PRIMARY KEY (`UserId`, `UserRoleId`)
);
 
CREATE TABLE IF NOT EXISTS `FacilityXRole` (
  `FacilityId` integer REFERENCES `Facilities` (`FacilityId`),
  `RoleId` integer REFERENCES `Roles` (`RoleId`),
  `DateIssued` timestamp,
  `ValidRange` timerange,
  PRIMARY KEY (`FacilityId`, `RoleId`)
);
 
-- exception layer: let a role access one particular door
CREATE TABLE IF NOT EXISTS `DoorXRole` (
  `RoleId` integer REFERENCES `Roles` (`RoleId`),
  `DoorId` integer REFERENCES `Doors` (`DoorId`),
  `Allowed` bool,
  `DateIssued` timestamp,
  `ValidRange` timerange,
  PRIMARY KEY (`RoleId`, `DoorId`)
);
 
-- exception layer: let a user access one particular door
CREATE TABLE IF NOT EXISTS `UserXDoor` (
  `UserId` integer REFERENCES `Users` (`UserId`),
  `DoorId` integer REFERENCES `Doors` (`DoorId`),
  `Allowed` bool,
  `DateIssued` timestamp,
  `ValidRange` timerange,
  PRIMARY KEY (`UserId`, `DoorId`)
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