"""URL map of the Authorization app."""

from django.urls import path

from .views import UserProfileAPIView


urlpatterns = [
    path("user-profile", UserProfileAPIView.as_view(), name="user-profile"),
]
