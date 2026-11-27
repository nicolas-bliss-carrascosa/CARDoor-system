from django.shortcuts import render
from django.utils.timezone import localdate, localtime
from django.db.models import F, Count
from django.db.models.functions import TruncDate
from collections import Counter
from datetime import datetime, timedelta
from bokeh.plotting import figure
from bokeh.models import DatetimeTickFormatter
from bokeh.embed import components
from dataclasses import dataclass
from .models import Logs, LivenessLog
from .aggregates import MinGapSeconds
from facilities.models import Doors

MULITPLE_DOORS_ALERT = "Multiple door events in quick succession"
DEAD_DOOR_ALERT = "Door has not pinged central server"
MIN_GAP_THRESHOLD = 10  # seconds
TIME_RANGE = 14
PING_INTERVAL_MINS = 60

@dataclass(frozen=True)
class Alert:
    obj_id: int
    name: str
    message: str 

def find_attempts_alerts(date):
    too_many_attempts_rows = (
        Logs.objects.filter(date__date=date)
        .values("user_id", user_name=F("user__name"))
        .annotate(min_gap=MinGapSeconds("date", "door"))
        .filter(min_gap__isnull=False, min_gap__lt=MIN_GAP_THRESHOLD)
        .order_by("min_gap")
    )

    return [
        Alert(
            obj_id=r["user_id"],
            name=r["user_name"] or f"User {r['user_id']}",
            message = MULITPLE_DOORS_ALERT
        )
        for r in too_many_attempts_rows
    ]

def find_door_alerts(date):
    cutoff = localtime() - timedelta(minutes=PING_INTERVAL_MINS)

    recent_door_ids = LivenessLog.objects.filter(date__gte=cutoff).values("door_id")
    dead_doors = Doors.objects.exclude(pk__in=recent_door_ids)

    return [
        Alert(obj_id=d.pk, name=d.name, message=DEAD_DOOR_ALERT) 
        for d in dead_doors
    ]

def find_alerts(date):
    return find_door_alerts(date) + find_attempts_alerts(date)


def count_dead_doors_per_day(end_date=None, days=14):
    end_date = end_date or localdate()
    start_date = end_date - timedelta(days=days - 1)
    expected_pings = (24 * 60) // PING_INTERVAL_MINS

    rows = (
        LivenessLog.objects
        .filter(date__date__gte=start_date, date__date__lte=end_date)
        .annotate(day=TruncDate("date"))
        .values("day", "door_id")
        .annotate(n=Count("date"))
        .order_by()
    )

    healthy = Counter(row["day"] for row in rows if row["n"] >= expected_pings)

    total_doors = Doors.objects.count()
    return [
        total_doors - healthy[start_date + timedelta(days=i)]
        for i in range(days)
    ]

def weekly_alerts(start_date):

    attempts = [ (
        Logs.objects.filter(date__date=start_date + timedelta(days=i))
        .values("user_id", user_name=F("user__name"))
        .annotate(min_gap=MinGapSeconds("date", "door"))
        .filter(min_gap__isnull=False, min_gap__lt=MIN_GAP_THRESHOLD)
        .count()
    )
    for i in range(TIME_RANGE)
    ]
    
    doors = count_dead_doors_per_day(days=TIME_RANGE)
    return [x + y for x, y in zip(doors, attempts)]


def dashboard(request):
    today = localdate()
    one_week_ago = today - timedelta(days=TIME_RANGE - 1)

    interactions_today = Logs.objects.filter(date=today)
    alerts_today = find_alerts(localdate())

    this_week = [
        one_week_ago + timedelta(days=i)
        for i in range(TIME_RANGE)
    ]

    count_weekly_interactions = [
        Logs.objects.filter(date__gte=one_week_ago + timedelta(i)).count()
        for i in range(TIME_RANGE)
    ]
    count_weekly_alerts = weekly_alerts(one_week_ago)
    breakpoint()
    one_day = timedelta(days=0.9).total_seconds() * 1000
    p = figure(title=f"Alerts and Interactions, {one_week_ago.month}/{one_week_ago.day}-{today.month}/{today.day}", 
                x_axis_label="Date", 
                y_axis_label="Number",
                x_axis_type="datetime",
                sizing_mode="stretch_width")
    p.vbar(x=this_week, top=count_weekly_interactions, legend_label="Interactions", width=one_day, bottom=0, color="#468bfa")
    p.line(x=this_week, y=count_weekly_alerts, legend_label="Alerts", line_width=3, color="#fa3737")

    p.xaxis.formatter = DatetimeTickFormatter(
        days="%b %d",
        months="%b %Y",
        years="%Y",
    )
    script, div = components(p)

    return render(request, "dashboard/dashboard.html", {
        "interactions": interactions_today,
        "alerts": alerts_today,
        "script": script,
        "div": div
    })
