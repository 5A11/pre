"""URL map of the DataAccess app."""

from django.urls import path

from .views import (
    DataAccessAPIView,
    DataAccessCreateAPIView,
    DataAccessDownloadAPIView,
    DataAccessGrantedListAPIView,
    DataAccessOwnedListAPIView,
)


urlpatterns = [
    path("<int:pk>", DataAccessAPIView.as_view(), name="data-access"),
    path("create", DataAccessCreateAPIView.as_view(), name="data-access-create"),
    path("owned", DataAccessOwnedListAPIView.as_view(), name="data-access-owned-list"),
    path(
        "granted",
        DataAccessGrantedListAPIView.as_view(),
        name="data-access-granted-list",
    ),
    path(
        "<int:pk>/download",
        DataAccessDownloadAPIView.as_view(),
        name="data-access-download",
    ),
]
