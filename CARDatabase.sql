-- TODO: fix timestamp / timerange / whatever

CREATE TABLE `Users` (
  `UserId` integer PRIMARY KEY,
  `Name` varchar(255),
  `DateIssued` timestamp
);

CREATE TABLE `Roles` (
  `RoleId` integer PRIMARY KEY,
  `description` varchar(255)
);

CREATE TABLE `Facilities` (
  `FacilityId` integer PRIMARY KEY,
  `Name` varchar(255)
);

CREATE TABLE `Doors` (
  `DoorId` integer PRIMARY KEY,
  `FacilityId` integer NOT NULL,
  `ExpectedState` bool -- standin for more complicated enum !
);

CREATE TABLE `UserXRole` (
  `UserId` integer,
  `UserRoleId` integer,
  `DateIssued` timestamp, 
  PRIMARY KEY (`UserId`, `UserRoleId`)
);

CREATE TABLE `FacilityXRole` (
  `FacilityId` integer,
  `RoleId` integer,
  `DateIssued` timestamp,
  `ValidRange` timerange,
  PRIMARY KEY (`FacilityId`, `RoleId`)
);

-- exception layer: let a role access one particular door
CREATE TABLE `DoorXRole` (
  `RoleId` integer,
  `DoorId` integer,
  `Allowed` bool,     
  `DateIssued` timestamp,
  `ValidRange` timerange,
  PRIMARY KEY (`RoleId`, `DoorId`)
);

-- exception layer: let a user access one particular door
CREATE TABLE `UserXDoor` (
  `UserId` integer,
  `DoorId` integer,
  `Allowed` bool,
  `DateIssued` timestamp,
  `ValidRange` timerange,
  PRIMARY KEY (`UserId`, `DoorId`)
);

-- this is appendonly
CREATE TABLE `Logs` (
  `DoorId` integer,
  `UserId` integer,
  `Date` timestamp,
  `Result` bool,
  PRIMARY KEY (`DoorId`, `UserId`, `Date`)
);

-- this is appendonly
CREATE TABLE `LivenessLog` (
  `DoorId` integer,
  `Date` timestamp,
  `Sent` str,
  PRIMARY KEY (`DoorId`, `Date`)
);

ALTER TABLE `UserXRole` ADD FOREIGN KEY (`UserId`) REFERENCES `Users` (`UserId`);

ALTER TABLE `UserXRole` ADD FOREIGN KEY (`UserRoleId`) REFERENCES `Roles` (`RoleId`);

ALTER TABLE `Doors` ADD FOREIGN KEY (`FacilityId`) REFERENCES `Facilities` (`FacilityId`);

ALTER TABLE `FacilityXRole` ADD FOREIGN KEY (`FacilityId`) REFERENCES `Facilities` (`FacilityId`);

ALTER TABLE `FacilityXRole` ADD FOREIGN KEY (`RoleID`) REFERENCES `Roles` (`RoleId`);

ALTER TABLE `DoorXRole` ADD FOREIGN KEY (`DoorId`) REFERENCES `Doors` (`DoorId`);

ALTER TABLE `DoorXRole` ADD FOREIGN KEY (`RoleId`) REFERENCES `Roles` (`RoleId`);

ALTER TABLE `UserXRole` ADD FOREIGN KEY (`DoorId`) REFERENCES `Doors` (`DoorId`);

ALTER TABLE `UserXRole` ADD FOREIGN KEY (`UserId`) REFERENCES `Users` (`UserId`);

ALTER TABLE `Logs` ADD FOREIGN KEY (`DoorId`) REFERENCES `Doors` (`DoorId`);

ALTER TABLE `Logs` ADD FOREIGN KEY (`UserId`) REFERENCES `Users` (`UserId`);

ALTER TABLE `LivenessLog` ADD FOREIGN KEY (`DoorId`) REFERENCES `Doors` (`DoorId`);
