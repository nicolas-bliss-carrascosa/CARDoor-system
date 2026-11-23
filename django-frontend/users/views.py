from django.http import Http404
from django.shortcuts import render
from .models import Users, Roles, UserXRole
from facilities.models import UserXDoor, Doors
from dashboard.models import Logs
from collections import defaultdict
from django.db.models import OuterRef, Subquery, F
from django.shortcuts import get_object_or_404, render
from django.contrib import messages
from django.utils import timezone
from django.db import transaction
# Create your views here.
def users(request):
    if request.method == "POST":
        action = request.POST.get("action")
        
        if action == "remove":
            user = get_object_or_404(Users, id=request.POST.get("user_id"))
            try:
                with transaction.atomic():
                    UserXDoor.objects.filter(user=user).delete()
                    UserXRole.objects.filter(user=user).delete()
                    user.delete()
            except Exception as e:
                print(repr(e))
                messages.error(request, "Could not delete the user. Nothing was changed.")
        elif action == "add_role":
            try:
                with transaction.atomic():
                    role_name = request.POST.get('role_name')
                    new_role = Roles.objects.create(description=role_name)
            except Exception as e:
                print(repr(e))
                messages.error(request, "Could not create the role.")
        elif action == "add_user":
            try:
                with transaction.atomic():
                    user_name = request.POST.get('user_name')
                    user_roles = request.POST.getlist('roles')
                    new_user = Users.objects.create(name=user_name, date_issued=timezone.now())
                    new_user.roles.set(user_roles)
            except Exception as e:
                print(repr(e))
                messages.error(request, "Could not create the user.")


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

def find_interactions(user_data):

    logs = (
        Logs.objects.filter(user=user_data)
        .annotate(door_name=F("door__name"), facility_name=F("door__facility__name"))
        .order_by("-date")
    )
    return logs


def user_detail(request, id):
    user = get_object_or_404(Users, id=id)

    if request.method == "POST":
        action = request.POST.get("action")

        if action == "remove":
            item_id = request.POST.get("item_id")

            deleted, _ = UserXRole.objects.filter(user=user, role_id=item_id).delete()

            if deleted:
                messages.success(request, "Removed.")
            else:
                messages.error(request, "Nothing was removed.")

        elif action == "add_role":
            role = get_object_or_404(Roles, id=request.POST.get("role"))
            _, created = UserXRole.objects.get_or_create(
                user=user, role=role,
                defaults={"date_issued": timezone.now()},
            )
            if created:
                messages.success(request, f"Added role {role.description}.")
            else:
                messages.info(request, "User already has that role.")


        else:
            messages.error(request, "Unknown action.")


    user_role_ids = find_role_ids(user)
    included_roles = Roles.objects.filter(id__in=user_role_ids)
    other_roles = Roles.objects.exclude(id__in=user_role_ids)
    interactions = find_interactions(user)

   
    return render(request, "users/user_detail.html", {
        "user": user,
        "includedRoles": included_roles,
        "otherRoles": other_roles,
        "interactions": interactions
    })