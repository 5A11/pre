"""Module with helper methods of Data accesses app."""

from django.conf import settings
from django.contrib.auth import get_user_model
from typing import Tuple

from pre.api.delegatee import DelegateeAPI
from pre.api.delegator import DelegatorAPI
from pre.contract.cosmos_contracts import DelegatorContract, ContractQueries
from pre.crypto.umbral_crypto import UmbralCrypto

from .models import DataAccess


User = get_user_model()


class DelegatorSDK:
    """Delegator SDK class."""

    def __init__(self, user: User):
        """Initialize DelegatorSDK."""
        self.delegator = self._get_delegator_api(user)

    @staticmethod
    def _get_delegator_api(user: User) -> DelegatorAPI:
        """
        Get DelegatorAPI for the user.

        :param user: User object.

        :return: DelegatorAPI object.
        """
        encryption_key = user.userprofile.encryption
        ledger_key = user.userprofile.ledger
        # Is that correct to transform string to bytes here?
        delegator_priv_key = UmbralCrypto.load_key(bytes(encryption_key, "utf-8"))

        delegator_contract = DelegatorContract(
            settings.LEDGER, settings.CONTRACT_ADDRESS
        )
        delegator_ledger_crypto = settings.LEDGER.load_crypto_from_str(ledger_key)

        return DelegatorAPI(
            delegator_priv_key,
            delegator_ledger_crypto,
            delegator_contract,
            settings.IPFS_STORAGE,
            settings.UMBRAL_CRYPTO,
        )

    def add_data(self, data: bytes) -> str:
        """
        Add data via PRE SDK.

        :param data: bytes data.

        :return: str hash_id of the added data.
        """
        data_id = self.delegator.add_data(data)
        return data_id

    def grant_access_via_public_key(self, data_id: str, public_key: bytes) -> None:
        """
        Grant access to the data via PRE SDK.

        :param data_id: the data_id of the object that reader needs to have access to.
        :param public_key: bytes public key of the reader.

        :return: None
        """
        self.delegator.grant_access(
            hash_id=data_id,
            delegatee_pubkey_bytes=public_key,
            threshold=settings.THRESHOLD,
        )

    def grant_access(self, data_access_obj: DataAccess, reader: User) -> None:
        """
        Grant access to the data via PRE SDK.

        :param data_access_obj: the object that reader needs to have access to.
        :param reader: User object that requests an access.

        :return: None
        """
        encryption_key = reader.userprofile.encryption
        delegatee_priv_key = UmbralCrypto.load_key(bytes(encryption_key, "utf-8"))
        self.grant_access_via_public_key(
            data_access_obj.data_id, bytes(delegatee_priv_key.public_key)
        )


class DelegateeSDK:
    """Delegatee SDK class."""

    def __init__(self, user: User):
        """Initialize DelegateeSDK."""
        (
            self.delegatee_api,
            self.query_contract,
        ) = self._get_delegatee_api_and_query_contract(user)

    @staticmethod
    def _get_delegatee_api_and_query_contract(
        user: User,
    ) -> Tuple[DelegateeAPI, ContractQueries]:
        """
        Get DelegateeAPI object.

        :param user: delegatee user.

        :return: DelegateeAPI object.
        """
        delegatee_priv_key = user.userprofile.encryption
        query_contract = ContractQueries(settings.LEDGER, settings.CONTRACT_ADDRESS)
        return (
            DelegateeAPI(
                delegatee_priv_key,
                query_contract,
                settings.IPFS_STORAGE,
                settings.UMBRAL_CRYPTO,
            ),
            query_contract,
        )

    def get_data(self, data_access_obj: DataAccess) -> bytes:
        """
        Get data by Data ID.

        :param data_id: str data ID.

        :return: bytes data content.
        :raises ValueError: if could not query data entry.
        """
        data_id = data_access_obj.data_id
        data_entry = self.query_contract.get_data_entry(data_id)
        if not data_entry:
            raise ValueError("Couldn't query data entry of data id from contract")

        return self.delegatee_api.read_data(data_id, data_entry.pubkey)
