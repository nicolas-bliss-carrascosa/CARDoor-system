from django.shortcuts import render

# Create your views here.

# Every group that exists, used to fill the "add group" dropdown.
ALL_GROUPS = [
    {"id": 1, "name": "Employee"},
    {"id": 2, "name": "Security"},
    {"id": 3, "name": "Night Staff"},
    {"id": 4, "name": "Contractor"},
    {"id": 5, "name": "Janitorial"},
]
 
FACILITIES = {
    1: {
        "id": 1,
        "name": "Headquarters",
        "groups": [
            {"id": 1, "name": "Employee"},
            {"id": 2, "name": "Security"},
            {"id": 4, "name": "Contractor"},
        ],
        "doors": [
            {"id": 1, "name": "Main lobby", "allowed" : True},
            {"id": 2, "name": "Floor 2", "allowed" : True},
            {"id": 3, "name": "Server room", "allowed" : False},
            {"id": 4, "name": "Executive suite", "allowed" : False},
            {"id": 5, "name": "Loading dock", "allowed" : False},
        ],
    },
    2: {
        "id": 2,
        "name": "Warehouse",
        "groups": [
            {"id": 1, "name": "Employee"},
            {"id": 2, "name": "Security"},
            {"id": 3, "name": "Night Staff"},
        ],
        "doors": [
            {"id": 6, "name": "Front office", "allowed" : True},
            {"id": 7, "name": "Warehouse floor", "allowed" : False},
            {"id": 8, "name": "Loading bay", "allowed" : False},
            {"id": 9, "name": "Cold storage", "allowed" : True},
        ],
    },
    3: {
        "id": 4,
        "name": "Annex",
        "groups": [],
        "doors": [],
    },
}
 
 
def available_groups(facility):
    """Groups not yet attached to this facility (for the add-group dropdown)."""
    taken = {g["id"] for g in facility["groups"]}
    return [g for g in ALL_GROUPS if g["id"] not in taken]
 

def facilities(request):
    return render(request, "facilities/facilities.html", {"facilities": FACILITIES})

def facility_detail(request, id):
    facility_data = FACILITIES.get(id)
    if not facility_data:
        raise Http404('Facility does not exist!')
    return render(request, 'facilities/facility_detail.html', {"facility": facility_data})