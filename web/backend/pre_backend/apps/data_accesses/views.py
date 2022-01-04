"""Views of the Data Accesses app."""

# from rest_framework.filters import OrderingFilter, SearchFilter
from rest_framework.generics import CreateAPIView, ListAPIView, RetrieveAPIView
from rest_framework.permissions import IsAuthenticated

from .models import DataAccess
from .serializers import DataAccessSerializer


class DataAccessListAPIView(ListAPIView):
    """API View to list ordered DataAccess objects."""

    serializer_class = DataAccessSerializer
    permission_classes = (
        IsAuthenticated,
    )  # TODO: add custom permission class to allow access to only owned data
    queryset = DataAccess.objects.all()

    # TODO: uncomment next if filtering is needed
    # filter_backends = (OrderingFilter, SearchFilter, )
    # search_fields = ("name",)
    # ordering = ("data_id",)
    # ordering_fields = ("data_id",)


class DataAccessAPIView(RetrieveAPIView):
    """API View to retreive DataAccess."""

    serializer_class = DataAccessSerializer
    permission_classes = (IsAuthenticated,)
    queryset = DataAccess.objects.all()

    # TODO: use this method if object should be taken by specific params (not by ID)
    # def get_object(self):
    #     """Get object by specific params."""


class DataAccessCreateAPIView(CreateAPIView):
    """API View to create DataAccess."""

    serializer_class = DataAccessSerializer
    permission_classes = (IsAuthenticated,)
    permission_classes = (IsAuthenticated,)
