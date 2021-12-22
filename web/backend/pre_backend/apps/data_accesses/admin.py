from django.contrib import admin

from .models import DataAccess


class DataAccessAdmin(admin.ModelAdmin):
    """Customizing DataAccess admin panel view."""

    list_display = ("owner", "data_id")


admin.site.register(DataAccess, DataAccessAdmin)
