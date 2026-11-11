from django.urls import path
from . import views

urlpatterns = [
    path('', views.facilities, name="facilities"),
    path('<int:id>', views.facility_detail, name="facility_detail")
]