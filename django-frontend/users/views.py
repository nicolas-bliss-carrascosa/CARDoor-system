from django.http import Http404
from django.shortcuts import render

MOCK_USERS = {
    101: {
        'id': 101,
        'name': 'alice',
        'role': 'professor'
    },
    102: {
        'id': 102,
        'name': 'bob',
        'role': 'student'
    }
}

# Create your views here.
def users(request):
    return render(request, "users/users.html", {"users": MOCK_USERS})

def user_detail(request, id):
    user_data = MOCK_USERS.get(id)
    if not user_data:
        raise Http404('User does not exist')
    return render(request, "users/user_detail.html", {"user": user_data})