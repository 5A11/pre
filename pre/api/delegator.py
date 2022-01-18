from typing import IO, List, Union

from pre.common import DelegationState, HashID, PrivateKey
from pre.contract.base_contract import AbstractDelegatorContract
from pre.crypto.base_crypto import AbstractCrypto
from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.storage.base_storage import AbstractStorage


class DelegatorAPI:
    """Delegator API to add encrypted data and grant access to a specific delegatee."""

    _contract: AbstractDelegatorContract

    def __init__(
        self,
        encryption_private_key: PrivateKey,
        ledger_crypto: AbstractLedgerCrypto,
        contract: AbstractDelegatorContract,
        storage: AbstractStorage,
        crypto: AbstractCrypto,
    ):
        """
        Init api isntance.

        :param encryption_private_key: PrivateKey,
        :param ledger_crypto: ledger private key instance,
        :param contract: instance of delegator contract implementation,
        :param storage: instance of abstract storage implementation,
        :param crypto: instance of abstract crypto implementation,
        """
        self._encryption_private_key = encryption_private_key
        self._ledger_crypto = ledger_crypto
        self._contract = contract
        self._storage = storage
        self._crypto = crypto

    @property
    def _encryption_public_key(self) -> bytes:
        """
        Get encryption public key.

        :return: bytes
        """
        return bytes(self._encryption_private_key.public_key)

    def add_data(self, data: Union[bytes, IO]) -> HashID:
        """
        Register data to be encrypted and published on the storage

        :param data: bytes

        :return: str, hash id of the encrypteed data published
        """
        encrypted_data, capsule = self._crypto.encrypt(
            data, self._encryption_private_key.public_key
        )
        hash_id = self._storage.store_encrypted_data(encrypted_data)
        self._contract.add_data(
            delegator_private_key=self._ledger_crypto,
            delegator_pubkey_bytes=self._encryption_public_key,
            hash_id=hash_id,
            capsule=capsule,
        )
        return hash_id

    def get_selected_proxies_for_delegation(
        self,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        """
        Get list of proxies pub keys for delegation

        :param delegatee_pubkey_bytes: reader public key in bytes

        :return: List[bytes], list of proxies public keys in bytes
        """
        return self._contract.get_selected_proxies_for_delegation(
            self._encryption_public_key,
            delegatee_pubkey_bytes,
        )

    def _set_delegation(
        self,
        delegatee_pubkey_bytes: bytes,
        threshold: int,
    ):
        """
        Set permanent delegation for a specific delegatee.

        :param delegatee_pubkey_bytes: reader public key in bytes
        :param threshold: int
        """

        proxies_list = self.get_selected_proxies_for_delegation(delegatee_pubkey_bytes)
        if not proxies_list:
            # request proxies from contract
            proxies_list = self._contract.request_proxies_for_delegation(
                delegator_private_key=self._ledger_crypto,
                delegator_pubkey_bytes=self._encryption_public_key,
                delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            )

        if not proxies_list:
            raise ValueError("proxies_list can not be empty")

        delegations = self._crypto.generate_delegations(
            threshold=threshold,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            proxies_pubkeys_bytes=proxies_list,
            delegator_private_key=self._encryption_private_key,
        )
        self._contract.add_delegations(
            delegator_private_key=self._ledger_crypto,
            delegator_pubkey_bytes=self._encryption_public_key,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            delegations=delegations,
        )

    def grant_access(
        self,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        threshold: int,
    ):
        """
        Grant access for specific data for specific delegatee.

        :param hash_id: str, hash id of the encrypted data published
        :param delegatee_pubkey_bytes: reader public key in bytes
        :param threshold: int
        """
        delegation_state_response = self._contract.get_delegation_status(
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        )

        if delegation_state_response.delegation_state == DelegationState.non_existing:
            self._set_delegation(
                delegatee_pubkey_bytes=delegatee_pubkey_bytes,
                threshold=threshold,
            )

            # Update state to get correct minimum_request_reward
            delegation_state_response = self._contract.get_delegation_status(
                delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
                delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            )

        # DelegationState.Active
        self._contract.request_reencryption(
            delegator_private_key=self._ledger_crypto,
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            hash_id=hash_id,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            stake_amount=delegation_state_response.minimum_request_reward,
        )
