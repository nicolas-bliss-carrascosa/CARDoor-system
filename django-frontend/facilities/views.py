from django.contrib import messages
from django.db import transaction
from django.db.models import Q
from django.shortcuts import get_object_or_404, render
from django.utils import timezone

from users.models import Roles
from .models import Doors, Facilities, FacilityXRole


def roles(facility):
    role_ids = FacilityXRole.objects.filter(facility=facility).values_list('role_id', flat=True)
    included = Roles.objects.filter(id__in=role_ids)
    excluded = Roles.objects.filter(~Q(id__in=role_ids))
    return included, excluded


def facilities(request):
    if request.method == "POST":
        action = request.POST.get("action")

        if action == "remove":
            facility = get_object_or_404(Facilities, id=request.POST.get("facility_id"))
            try:
                # on_delete=CASCADE removes its doors, role links and door grants
                facility.delete()
            except Exception as e:
                print(repr(e))
                messages.error(request, "Could not delete the facility. Nothing was changed.")

        elif action == "add_facility":
            try:
                with transaction.atomic():
                    facility_name = request.POST.get("facility_name")
                    role_ids = request.POST.getlist("roles")
                    new_facility = Facilities.objects.create(name=facility_name)
                    now = timezone.now()
                    FacilityXRole.objects.bulk_create([
                        FacilityXRole(facility=new_facility, role_id=rid, date_issued=now)
                        for rid in role_ids
                    ])
            except Exception as e:
                print(repr(e))
                messages.error(request, "Could not create the facility.")

    return render(request, "facilities/facilities.html", {
        "facilities": Facilities.objects.all(),
        "roles": Roles.objects.all(),
    })


def facility_detail(request, id):
    facility_data = get_object_or_404(Facilities, id=id)

    if request.method == "POST":
        action = request.POST.get("action")

        if action == "remove":
            item_id = request.POST.get("item_id")
            deleted, _ = FacilityXRole.objects.filter(facility=facility_data, role_id=item_id).delete()

            if deleted:
                messages.success(request, "Removed.")
            else:
                messages.error(request, "Nothing was removed.")

        elif action == "add_role":
            role = get_object_or_404(Roles, id=request.POST.get("role"))
            _, created = FacilityXRole.objects.get_or_create(
                facility=facility_data, role=role,
                defaults={"date_issued": timezone.now()},
            )

            if created:
                messages.success(request, f"Added role {role.description}.")
            else:
                messages.info(request, "That role already has access.")

        elif action == "add_door":
            name = (request.POST.get("name") or "").strip()
            if name:
                Doors.objects.create(facility=facility_data, name=name)
                messages.success(request, f"Added door {name}.")
            else:
                messages.error(request, "Door name can't be blank.")

        elif action == "remove_door":
            # scoped to this facility so a door id from elsewhere can't be removed here
            deleted, _ = Doors.objects.filter(
                facility=facility_data, id=request.POST.get("item_id")
            ).delete()

            if deleted:
                messages.success(request, "Door removed.")
            else:
                messages.error(request, "Nothing was removed.")

        elif action == "toggle_door":
            door = get_object_or_404(Doors, id=request.POST.get("door_id"), facility=facility_data)
            door.expected_state = not door.expected_state
            door.save(update_fields=["expected_state"])
            messages.success(request, f"{door.name} is now {'unlocked' if door.expected_state else 'locked'}.")

        else:
            messages.error(request, "Unknown action.")

    # computed after the POST handling so the page reflects the change
    included_roles, excluded_roles = roles(facility_data)

    return render(request, 'facilities/facility_detail.html', {
        "facility": facility_data,
        "includedRoles": included_roles,
        "excludedRoles": excluded_roles,
        "doors": facility_data.doors.order_by("id"),
    })