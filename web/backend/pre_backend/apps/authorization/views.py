"""Views of the Data Accesses app."""

from django.contrib.auth import get_user_model
from django.http import Http404

from rest_framework.generics import RetrieveAPIView, RetrieveUpdateAPIView
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response

from .serializers import UserProfileSerializer


User = get_user_model()


class UserProfileAPIView(RetrieveUpdateAPIView):
    """API View to retreive and update UserProfile."""

    serializer_class = UserProfileSerializer
    permission_classes = (IsAuthenticated,)

    def get_object(self):
        """Get object by specific params."""
        user = self.request.user
        if not hasattr(user, "userprofile"):
            raise Http404()
        return user.userprofile


class UsernamesAPIView(RetrieveAPIView):
    """API view to get list of usernames."""

    permission_classes = (IsAuthenticated,)
    
    def get(self, request):
        """Handle GET request."""
        return Response(User.objects.all().values_list("username", flat=True))
