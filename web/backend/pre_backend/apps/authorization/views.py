"""Views of the Data Accesses app."""

from django.http import Http404

from rest_framework.generics import RetrieveUpdateAPIView
from rest_framework.permissions import IsAuthenticated

from .serializers import UserProfileSerializer


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
