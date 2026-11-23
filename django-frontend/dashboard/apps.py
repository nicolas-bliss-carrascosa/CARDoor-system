from django.apps import AppConfig
from django.db.backends.signals import connection_created

def register_udafs(sender, connection, **kwargs):
    if connection.vendor == "sqlite":
        from .aggregates import _MinGap
        connection.connection.create_aggregate("MIN_GAP", 2, _MinGap)

class DashboardConfig(AppConfig):
    default_auto_field = 'django.db.models.BigAutoField'
    name = 'dashboard'
    def ready(self):
        connection_created.connect(register_udafs)