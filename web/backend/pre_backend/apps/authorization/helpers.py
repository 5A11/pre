"""Helpers of Authorization app."""

from .models import UserProfile


def create_user_profile(user, encryption: str, ledger: str):
    """
    Create UserProfile for user.

    :param user: user object.
    :param encryption: str encryption.
    :param ledger: ste ledger.

    :return: None.
    """
    UserProfile.objects.create(user=user, encryption=encryption, ledger=ledger)
