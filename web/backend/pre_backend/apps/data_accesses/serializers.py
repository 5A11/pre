"""Serializers of the Data Accesses app."""

from rest_framework import serializers
from rest_framework.exceptions import APIException

from .helpers import DelegatorSDK
from .models import DataAccess


class DataAccessCreateSerializer(serializers.ModelSerializer):
    """DataAccess Create serializer."""

    file = serializers.FileField(write_only=True)

    class Meta:
        """DataAccess Create serializer setup."""

        model = DataAccess
        fields = ("file",)

    def create(self, validated_data):
        """Create new DataAccess object."""
        user = self.context["request"].user
        validated_data["owner"] = user
        file_obj = validated_data.pop("file")
        try:
            delegator_sdk = DelegatorSDK(user)
            validated_data["data_id"] = delegator_sdk.add_data(file_obj.read())
        except Exception as e:  # TODO: handle the exact expected exception and raise a proper status
            raise APIException(f"Failed to create a new DataAccess: {str(e)}")

        return DataAccess.objects.create(**validated_data)


class DataAccessSerializer(serializers.ModelSerializer):
    """DataAccess serializer."""

    reader_public_key = serializers.CharField(write_only=True, required=False)

    class Meta:
        """DataAccess serializer setup."""

        model = DataAccess
        fields = (
            "id",
            "owner",
            "readers",
            "reader_public_key",
        )
        read_only_fields = ("owner",)

    def update(self, instance, validated_data):
        """Update DataAccess object."""
        user = self.context["request"].user
        readers = validated_data.pop("readers")
        reader_public_key = validated_data.pop("reader_public_key", None)
        try:
            delegator_sdk = DelegatorSDK(user)
            for reader in readers:
                delegator_sdk.grant_access(instance, reader)

            # For unregistered readers
            if reader_public_key:
                delegator_sdk.grant_access_via_public_key(
                    instance.data_id, bytes(reader_public_key, "utf-8")
                )

        except Exception as e:  # TODO: handle the exact expected exception and raise a proper status
            raise APIException(f"Failed to update DataAccess: {str(e)}")
        else:
            instance.readers.set(readers)
            instance.save()

        return instance
