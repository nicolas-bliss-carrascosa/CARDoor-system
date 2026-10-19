from django.db import models
from facilities.models import Door
from users.models import User
class AppendOnlyModel(models.Model):
    """Rows can be inserted but never updated or deleted."""
 
    class Meta:
        abstract = True
 
    def save(self, *args, **kwargs):
        if not self._state.adding:
            raise ValueError(f"{type(self).__name__} is append-only")
        kwargs["force_insert"] = True
        super().save(*args, **kwargs)
 
    def delete(self, *args, **kwargs):
        raise ValueError(f"{type(self).__name__} is append-only")
 
 


class Log(AppendOnlyModel):
    door = models.ForeignKey(
        Door, on_delete=models.CASCADE, db_column="DoorId", related_name="logs"
    )
    user = models.ForeignKey(
        User, on_delete=models.CASCADE, db_column="UserId", related_name="logs"
    )
    date = models.DateTimeField(db_column="Date")
    result = models.BooleanField(null=True, db_column="Result")
 
    class Meta:
        db_table = "Logs"
        constraints = [
            models.UniqueConstraint(
                fields=["door", "user", "date"], name="uniq_log_door_user_date"
            ),
        ]
 
 
class LivenessLog(AppendOnlyModel):
    door = models.ForeignKey(
        Door, on_delete=models.CASCADE, db_column="DoorId", related_name="liveness_logs"
    )
    date = models.DateTimeField(db_column="Date")
    sent = models.TextField(null=True, blank=True, db_column="Sent")
 
    class Meta:
        db_table = "LivenessLog"
        constraints = [
            models.UniqueConstraint(
                fields=["door", "date"], name="uniq_livenesslog_door_date"
            ),
        ]
 
