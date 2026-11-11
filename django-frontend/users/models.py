from django.db import models

class Users(models.Model):
    id = models.IntegerField(primary_key=True, db_column="UserId")
    name = models.CharField(max_length=255, null=True, blank=True, db_column="Name")
    date_issued = models.DateTimeField(null=True, blank=True, db_column="DateIssued")
 
    roles = models.ManyToManyField("Roles", through="UserXRole", related_name="users")
 
    class Meta:
        db_table = "Users"
 
    def __str__(self):
        return self.name or f"User {self.pk}"
 
 
class Roles(models.Model):
    id = models.IntegerField(primary_key=True, db_column="RoleId")
    description = models.CharField(max_length=255, null=True, blank=True)
 
    class Meta:
        db_table = "Roles"
 
    def __str__(self):
        return self.description or f"Role {self.pk}"
 
 
class UserXRole(models.Model):
    user = models.ForeignKey(
        Users, on_delete=models.CASCADE, db_column="UserId", related_name="user_roles"
    )
    role = models.ForeignKey(
        Roles, on_delete=models.CASCADE, db_column="UserRoleId", related_name="role_users"
    )
    date_issued = models.DateTimeField(null=True, blank=True, db_column="DateIssued")
 
    class Meta:
        db_table = "UserXRole"
        constraints = [
            models.UniqueConstraint(fields=["user", "role"], name="uniq_userxrole"),
        ]
 
