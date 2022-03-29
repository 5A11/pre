"""Serializer utils."""

from django.contrib.auth import get_user_model

from rest_framework import serializers
from rest_framework.exceptions import ValidationError


User = get_user_model()


class UsernameRelatedField(serializers.RelatedField):
    """Serializer custom field to represent a user with username."""

    queryset = User.objects.all()

    def to_representation(self, value):
        """Get object representation."""
        return value.username

    def to_internal_value(self, username):
        """Get object internal value."""
        try:
            return User.objects.get(username=username)
        except User.DoesNotExist:
            raise ValidationError(f"User with username '{username}' does not exist.")
