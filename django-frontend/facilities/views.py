from django.shortcuts import render
from django.db.models import Q
from .models import Facilities, FacilityXRole
from users.models import Roles
# Create your views here.
 
 
def roles(facility):
    role_ids = FacilityXRole.objects.filter(facility=facility).values_list('role_id', flat=True)
    included = Roles.objects.filter(id__in=role_ids)
    excluded = Roles.objects.filter(~Q(id__in=role_ids))
    return included, excluded

def facilities(request):
    obj = Facilities.objects.all()
    return render(request, "facilities/facilities.html", {"facilities": obj})

def facility_detail(request, id):
    facility_data = Facilities.objects.get(id=id)
    if not facility_data:
        raise Http404('Facility does not exist!')
    included_roles, excluded_roles = roles(facility_data)

    return render(request, 'facilities/facility_detail.html', {"facility": facility_data, "roles": included_roles, "excluded_roles": excluded_roles})