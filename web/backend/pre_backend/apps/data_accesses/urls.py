"""URL map of the DataAccess app."""

from django.urls import path

from .views import DataAccessAPIView, DataAccessCreateAPIView, DataAccessListAPIView


urlpatterns = [
    path("", DataAccessListAPIView.as_view(), name="data-access-list"),
    path("<int:pk>", DataAccessAPIView.as_view(), name="data-access"),
    path("create", DataAccessCreateAPIView.as_view(), name="data-access-create"),
]
