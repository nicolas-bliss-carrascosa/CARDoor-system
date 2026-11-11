from django.http import Http404
from django.shortcuts import render
from .models import Users, Roles, UserXRole
from facilities.models import UserXDoor, Doors
from collections import defaultdict
from django.db.models import OuterRef, Subquery
from django.shortcuts import get_object_or_404, render

# Create your views here.
def users(request):
    users = list(Users.objects.all())
    pairs = list(UserXRole.objects.values_list("user_id", "role_id"))

    roles = Roles.objects.in_bulk({role_id for _, role_id in pairs})

    roles_by_user = defaultdict(list)
    for user_id, role_id in pairs:
        role = roles[role_id]
        roles_by_user[user_id].append({"id": role.id, "description": role.description})

    for u in users:
        u.role_list = roles_by_user[u.id]
    return render(request, "users/users.html", {"users": users, "roles": roles.values()})

def find_role_ids(user_data):
    return UserXRole.objects.filter(user=user_data).values_list('role_id', flat=True)

def find_door_ids(user_data):
    return UserXDoor.objects.filter(user=user_data).values_list('door_id', flat=True)


def user_detail(request, id):
    user = get_object_or_404(Users, id=id)

    user_role_ids = find_role_ids(user)
    included_roles = Roles.objects.filter(id__in=user_role_ids)
    other_roles = Roles.objects.exclude(id__in=user_role_ids)

    user_door_ids = find_door_ids(user)
    user_override = UserXDoor.objects.filter(user=user, door=OuterRef('pk'))

    overridden_doors = (
        Doors.objects
        .filter(id__in=user_door_ids)
        .select_related('facility')
        .annotate(allowed=Subquery(user_override.values('allowed')[:1]))
    )
    other_doors = Doors.objects.exclude(id__in=user_door_ids).select_related('facility')
    breakpoint()
    return render(request, "users/user_detail.html", {
        "user": user,
        "includedRoles": included_roles,
        "otherRoles": other_roles,
        "overriddenDoors": overridden_doors,
        "otherDoors": other_doors,
    })