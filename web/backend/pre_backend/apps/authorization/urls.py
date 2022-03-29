"""URL map of the Authorization app."""

from django.urls import path

from .views import UsernamesAPIView, UserProfileAPIView


urlpatterns = [
    path("user-profile", UserProfileAPIView.as_view(), name="user-profile"),
    path("usernames", UsernamesAPIView.as_view(), name="usernames"),
]
