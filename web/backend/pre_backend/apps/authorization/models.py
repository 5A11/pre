"""Models of Authorization app."""

from django.conf import settings
from django.db import models

from .constants import ENCRYPTION_MAX_LENGTH, LEDGER_MAX_LENGTH


class UserProfile(models.Model):
    """User profile model."""

    encryption = models.CharField(max_length=ENCRYPTION_MAX_LENGTH, unique=True)
    ledger = models.CharField(max_length=LEDGER_MAX_LENGTH, unique=True)

    user = models.OneToOneField(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        primary_key=True,
    )

    def __str__(self):
        """String representation of UserProfile object."""
        return f"User profile: {self.user.username}"
