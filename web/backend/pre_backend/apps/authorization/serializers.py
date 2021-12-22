"""Serializers of Authorization app."""

from rest_auth.registration.serializers import (
    RegisterSerializer as BaseRegisterSerializer,
)
from rest_framework import serializers

from .constants import (
    ENCRYPTION_MAX_LENGTH,
    ENCRYPTION_MIN_LENGTH,
    LEDGER_MAX_LENGTH,
    LEDGER_MIN_LENGTH,
)
from .helpers import create_user_profile


class RegisterSerializer(BaseRegisterSerializer):
    """Custom RegisterSerializer for user signup."""

    encryption = serializers.CharField(
        max_length=ENCRYPTION_MAX_LENGTH, min_length=ENCRYPTION_MIN_LENGTH
    )
    ledger = serializers.CharField(
        max_length=LEDGER_MAX_LENGTH, min_length=LEDGER_MIN_LENGTH
    )

    def custom_signup(self, request, user):
        """Perform actions on signup."""
        encryption, ledger = (
            self.validated_data.get("encryption"),
            self.validated_data.get("ledger"),
        )
        create_user_profile(user, encryption, ledger)
        user.save()
