from django.shortcuts import render

# Create your views here.
from django.shortcuts import render

def home(request):
    return render(request, "dashboard/home.html", {"name": "world"})

def dashboard(request):
    return render(request, "dashboard/dashboard.html")
    