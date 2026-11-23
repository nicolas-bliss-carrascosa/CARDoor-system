from django.shortcuts import render
from django.utils.timezone import localdate, localtime
from django.db.models import F
from datetime import datetime, timedelta
from bokeh.plotting import figure
from bokeh.models import DatetimeTickFormatter
from bokeh.embed import components
from dataclasses import dataclass
from .models import Logs
from .aggregates import MinGapSeconds

MULITPLE_DOORS_ALERT = "Multiple door events in quick succession"
MIN_GAP_THRESHOLD = 10  # seconds
TIME_RANGE = 14

@dataclass(frozen=True)
class Alert:
    user_id: int
    user_name: str
    time: datetime
    message: str 


def find_alerts(date):
    too_many_attempts_rows = (
        Logs.objects.filter(date__date=date)
        .values("user_id", user_name=F("user__name"))
        .annotate(min_gap=MinGapSeconds("date", "door"))
        .filter(min_gap__isnull=False, min_gap__lt=MIN_GAP_THRESHOLD)
        .order_by("min_gap")
    )

    attempts_alerts = [
        Alert(
            user_id=r["user_id"],
            user_name=r["user_name"] or f"User {r['user_id']}",
            message = MULITPLE_DOORS_ALERT
        )
        for r in too_many_attempts_rows
    ]

    return attempts_alerts

def weekly_alerts(start_date):

    return [ (
        Logs.objects.filter(date__date=start_date + timedelta(days=i))
        .values("user_id", user_name=F("user__name"))
        .annotate(min_gap=MinGapSeconds("date", "door"))
        .filter(min_gap__isnull=False, min_gap__lt=MIN_GAP_THRESHOLD)
        .count()
    )
    for i in range(TIME_RANGE)
    ]


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

    p = figure(title=f"Alerts and Interactions, {one_week_ago.month}/{one_week_ago.day}-{today.month}/{today.day}", 
                x_axis_label="Date", 
                y_axis_label="Number",
                x_axis_type="datetime",
                sizing_mode="stretch_width")
    p.vbar(x=this_week, top=count_weekly_interactions, legend_label="Interactions", width=0.8, bottom=0, color="blue")
    p.vbar(x=this_week, top=count_weekly_alerts, legend_label="Alerts", width=0.8, bottom=0, color="red")

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
