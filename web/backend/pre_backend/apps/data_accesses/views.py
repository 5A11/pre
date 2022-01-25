"""Views of the Data Accesses app."""

from django.http import FileResponse
from django.shortcuts import get_object_or_404

# from rest_framework.filters import OrderingFilter, SearchFilter
from rest_framework.exceptions import APIException
from rest_framework.generics import CreateAPIView, ListAPIView, RetrieveUpdateAPIView
from rest_framework.permissions import IsAuthenticated
from rest_framework.views import APIView

from .helpers import DelegateeSDK
from .models import DataAccess
from .permissions import IsOwner, IsReader
from .serializers import DataAccessCreateSerializer, DataAccessSerializer


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


class DataAccessAPIView(RetrieveUpdateAPIView):
    """API View to retreive DataAccess."""

    serializer_class = DataAccessSerializer
    permission_classes = (IsAuthenticated, IsOwner)
    queryset = DataAccess.objects.all()

    # TODO: use this method if object should be taken by specific params (not by ID)
    # def get_object(self):
    #     """Get object by specific params."""


class DataAccessCreateAPIView(CreateAPIView):
    """API View to create DataAccess."""

    serializer_class = DataAccessCreateSerializer
    permission_classes = (IsAuthenticated,)


class DataAccessDownloadAPIView(APIView):
    """API view to download data by reader."""

    permission_classes = (IsAuthenticated, IsReader)
    queryset = DataAccess.objects.all()

    def get_object(self, pk):
        return get_object_or_404(self.queryset, pk=pk)

    def get(self, request, pk, *args):
        """Get a data file to download."""
        obj = self.get_object(pk)
        try:
            delegatee_sdk = DelegateeSDK(request.user)
            data_content = delegatee_sdk.get_data(obj)
        except Exception as e:  # TODO: handle an exact exception class
            raise APIException(f"Failed to get data for downloading: {str(e)}")
        else:
            return FileResponse(data_content)
