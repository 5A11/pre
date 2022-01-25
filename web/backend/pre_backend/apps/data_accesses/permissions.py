"""Module with permissions of DataAccess app."""

from rest_framework import permissions


class IsReader(permissions.BasePermission):
    """A permission to check if user is reader of data."""

    message = "Only reader or owner is allowed."

    def has_object_permission(self, request, view, obj):
        """Check is user permitted to access the object."""
        return (request.user in obj.readers) or (obj.owner == request.user)


class IsOwner(permissions.BasePermission):
    """A permission to check if user is reader of data."""

    message = "Only owner is allowed."

    def has_object_permission(self, request, view, obj):
        """Check is user permitted to access the object."""
        return obj.owner == request.user
