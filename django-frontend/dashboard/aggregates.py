# aggregates.py
from django.db.models import Aggregate, FloatField

class MinGapSeconds(Aggregate):
    function = "MIN_GAP"
    output_field = FloatField()
    arity = 2

from datetime import datetime

class _MinGap:
    def __init__(self):
        self.events = []

    def step(self, value, door):
        if value is not None and door is not None:
            self.events.append((datetime.fromisoformat(value), door))

    def finalize(self):
        self.events.sort()
        gaps = [
            (b_t - a_t).total_seconds()
            for (a_t, a_d), (b_t, b_d) in zip(self.events, self.events[1:])
            if a_d != b_d
        ]
        return min(gaps) if gaps else None