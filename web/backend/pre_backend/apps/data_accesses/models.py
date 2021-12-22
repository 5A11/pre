"""Models of Data Access Rights app."""

from django.conf import settings
from django.db import models

from .constants import DATA_ID_MAX_LENGTH


class DataAccess(models.Model):
    """Data access model."""

    data_id = models.CharField(max_length=DATA_ID_MAX_LENGTH, unique=True)

    owner = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="data_accesses_owned",
    )
    readers = models.ManyToManyField(
        settings.AUTH_USER_MODEL, related_name="data_accesses_granted"
    )

    def __str__(self):
        """String representation of DataAccess object."""
        return self.data_id

    class Meta:
        verbose_name = "data access"
        verbose_name_plural = "data accesses"
