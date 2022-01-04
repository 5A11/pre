"""Serializers of the Data Accesses app."""

from rest_framework import serializers

from .models import DataAccess


class DataAccessSerializer(serializers.ModelSerializer):
    """DataAccess serializer."""

    owner = serializers.ReadOnlyField(source="owner.username")

    class Meta:
        """DataAccess list serializer setup."""

        model = DataAccess
        fields = (
            "id",
            "data_id",
            "owner",
            "readers",  # TODO: check if readers list should be visible to other readers
        )

    def create(self, validated_data):
        """Create new DataAccess object."""
        validated_data["owner"] = self.context["request"].user
        readers = validated_data.pop("readers")
        obj = DataAccess(**validated_data)
        obj.save()
        obj.readers.set(readers)
        return obj
