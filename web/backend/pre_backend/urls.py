"""pre_backend URL Configuration

The `urlpatterns` list routes URLs to views. For more information please see:
    https://docs.djangoproject.com/en/3.2/topics/http/urls/
Examples:
Function views
    1. Add an import:  from my_app import views
    2. Add a URL to urlpatterns:  path('', views.home, name='home')
Class-based views
    1. Add an import:  from other_app.views import Home
    2. Add a URL to urlpatterns:  path('', Home.as_view(), name='home')
Including another URLconf
    1. Import the include() function: from django.urls import include, path
    2. Add a URL to urlpatterns:  path('blog/', include('blog.urls'))
"""
from django.conf import settings
from django.contrib import admin
from django.conf.urls.static import static
from django.urls import include, path, re_path

from pre_backend.utils.views import ConfirmEmailAPIView, ForwardToFrontAPIView
from rest_auth.views import PasswordResetConfirmView  # type: ignore
from rest_auth.registration.views import VerifyEmailView


urlpatterns = [
    path("admin/", admin.site.urls),
    re_path(
        r"^rest-auth/registration/account-confirm-email/(?P<key>.+)/$",
        ConfirmEmailAPIView.as_view(),
        name="account_confirm_email",
    ),
    re_path(
        r"^account-confirm-email/",
        VerifyEmailView.as_view(),
        name="account_email_verification_sent",
    ),
    re_path(
        r"^password/reset/confirm/$",
        PasswordResetConfirmView.as_view(),
        name="rest_password_reset_confirm",
    ),
    path(
        "password-reset/<uidb64>/<token>/",
        ForwardToFrontAPIView.as_view(),
        name="password_reset_confirm",
    ),
    path("rest-auth/", include("rest_auth.urls")),
    path("rest-auth/registration/", include("rest_auth.registration.urls")),
    
    # Internal URLs
    path("authorization/", include("pre_backend.apps.authorization.urls")),
    path("data-accesses/", include("pre_backend.apps.data_accesses.urls")),
] + static(settings.MEDIA_URL, document_root=settings.MEDIA_ROOT)
