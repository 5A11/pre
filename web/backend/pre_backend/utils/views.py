"""Module with generic views."""

from django.conf import settings
from django.shortcuts import redirect
from rest_framework.generics import GenericAPIView


class ConfirmEmailAPIView(GenericAPIView):
    """API View to process email verification request."""

    def get(self, request, key):  # pragma: nocover
        """Process GET request."""
        url = "{}/confirm-email/{}".format(settings.FRONT_URL, key)
        return redirect(url)


class ForwardToFrontAPIView(GenericAPIView):
    """API View to forward request to front-end server."""

    def get(self, request, *args, **kwargs):  # pragma: nocover
        """Process GET request."""
        url = "{}{}".format(settings.FRONT_URL, request.path)
        return redirect(url)
