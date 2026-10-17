from django.db import models

# Create your models here.
class Cardholder(models.Model):
    userid = models.IntegerField()
    name = models.CharField(max_length=255)
    