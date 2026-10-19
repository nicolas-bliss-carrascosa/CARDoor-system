from django.db import models
from users.models import User, Role
class ValidRangeMixin(models.Model):
 
    valid_from = models.TimeField(null=True, blank=True, db_column="ValidFrom")
    valid_to = models.TimeField(null=True, blank=True, db_column="ValidTo")
 
    class Meta:
        abstract = True
 

class Facility(models.Model):
    id = models.IntegerField(primary_key=True, db_column="FacilityId")
    name = models.CharField(max_length=255, null=True, blank=True, db_column="Name")
 
    class Meta:
        db_table = "Facilities"
        verbose_name_plural = "facilities"
 
    def __str__(self):
        return self.name or f"Facility {self.pk}"
 
 
class Door(models.Model):
    id = models.IntegerField(primary_key=True, db_column="DoorId")
    facility = models.ForeignKey(
        Facility,
        on_delete=models.CASCADE,
        db_column="FacilityId",
        related_name="doors",
    )
    name = models.CharField(max_length=255, null=True, blank=True, db_column="Name")
    # stand-in for a more complicated enum
    expected_state = models.BooleanField(null=True, db_column="ExpectedState")
 
    class Meta:
        db_table = "Doors"
 
    def __str__(self):
        return self.name or f"Door {self.pk}"

class FacilityXRole(ValidRangeMixin):
    facility = models.ForeignKey(
        Facility,
        on_delete=models.CASCADE,
        db_column="FacilityId",
        related_name="facility_roles",
    )
    role = models.ForeignKey(
        Role,
        on_delete=models.CASCADE,
        db_column="RoleId",
        related_name="role_facilities",
    )
    date_issued = models.DateTimeField(null=True, blank=True, db_column="DateIssued")
 
    class Meta:
        db_table = "FacilityXRole"
        constraints = [
            models.UniqueConstraint(
                fields=["facility", "role"], name="uniq_facilityxrole"
            ),
        ]
 
 
# exception layer: let a role access one particular door
class DoorXRole(ValidRangeMixin):
    role = models.ForeignKey(
        Role, on_delete=models.CASCADE, db_column="RoleId", related_name="door_grants"
    )
    door = models.ForeignKey(
        Door, on_delete=models.CASCADE, db_column="DoorId", related_name="role_grants"
    )
    allowed = models.BooleanField(null=True, db_column="Allowed")
    date_issued = models.DateTimeField(null=True, blank=True, db_column="DateIssued")
 
    class Meta:
        db_table = "DoorXRole"
        constraints = [
            models.UniqueConstraint(fields=["role", "door"], name="uniq_doorxrole"),
        ]
 
 
# exception layer: let a user access one particular door
class UserXDoor(ValidRangeMixin):
    user = models.ForeignKey(
        User, on_delete=models.CASCADE, db_column="UserId", related_name="door_grants"
    )
    door = models.ForeignKey(
        Door, on_delete=models.CASCADE, db_column="DoorId", related_name="user_grants"
    )
    allowed = models.BooleanField(null=True, db_column="Allowed")
    date_issued = models.DateTimeField(null=True, blank=True, db_column="DateIssued")
 
    class Meta:
        db_table = "UserXDoor"
        constraints = [
            models.UniqueConstraint(fields=["user", "door"], name="uniq_userxdoor"),
        ]
 
