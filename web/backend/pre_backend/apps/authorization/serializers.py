"""Serializers of Authorization app."""

from django.utils.crypto import get_random_string

from rest_auth.registration.serializers import (
    RegisterSerializer as BaseRegisterSerializer,
)
from rest_framework import serializers

from .constants import (
    ENCRYPTION_MAX_LENGTH,
    LEDGER_MAX_LENGTH,
)
from .helpers import create_user_profile
from .models import UserProfile


class RegisterSerializer(BaseRegisterSerializer):
    """Custom RegisterSerializer for user signup."""

    def custom_signup(self, request, user):
        """Perform actions on signup."""
        # TODO: replace next with generating real encryption and ledger keys
        encryption, ledger = (
            get_random_string(ENCRYPTION_MAX_LENGTH),
            get_random_string(LEDGER_MAX_LENGTH),
        )
        create_user_profile(user, encryption, ledger)
        user.save()


class UserProfileSerializer(serializers.ModelSerializer):
    """UserProfile serializer."""

    class Meta:
        """DataAccess list serializer setup."""

        model = UserProfile
        fields = (
            "encryption",
            "ledger",
        )
