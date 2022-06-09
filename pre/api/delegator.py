from typing import IO, List, Optional, Union

from pre.common import DelegationState, HashID, PrivateKey, ProxyAvailability
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

    def get_available_proxies(
        self,
    ) -> List[ProxyAvailability]:
        """
        Get list of proxies pub keys and stake_amount for delegation

        :return: List[ProxyAvailability], list of proxies public keys in bytes and stake_amount
        """
        return self._contract.get_available_proxies()

    def _set_delegation(
        self,
        delegatee_pubkey_bytes: bytes,
        threshold: int,
        n_max_proxies: Optional[int] = None,
    ):
        """
        Set permanent delegation for a specific delegatee.

        :param delegatee_pubkey_bytes: reader public key in bytes
        :param threshold: int
        :param n_max_proxies: Optional[int]
        """

        proxies_list = self.get_available_proxies()
        if not proxies_list:
            raise ValueError("proxies_list can not be empty")

        # Sort descending by stake amount
        proxies_list.sort(reverse=True, key=lambda x: int(x.stake_amount))

        n_max_proxies = (
            n_max_proxies
            if n_max_proxies is not None and n_max_proxies < len(proxies_list)
            else len(proxies_list)
        )

        # Select up to n_max_proxies proxies with highest available stake
        proxy_pubkeys = [i.proxy_pubkey for i in proxies_list[0:n_max_proxies]]
        proxy_addresses = [i.proxy_addr for i in proxies_list[0:n_max_proxies]]

        delegations = self._crypto.generate_delegations(
            threshold=threshold,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            proxies_pubkeys_bytes=proxy_pubkeys,
            proxies_addresses=proxy_addresses,
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
        n_max_proxies: int,
    ):
        """
        Grant access for specific data for specific delegatee.

        :param hash_id: str, hash id of the encrypted data published
        :param delegatee_pubkey_bytes: reader public key in bytes
        :param threshold: int
        :param n_max_proxies: int
        """
        delegation_state_response = self._contract.get_delegation_status(
            delegator_pubkey_bytes=bytes(self._encryption_private_key.public_key),
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
        )

        if delegation_state_response.delegation_state == DelegationState.non_existing:
            self._set_delegation(
                delegatee_pubkey_bytes=delegatee_pubkey_bytes,
                threshold=threshold,
                n_max_proxies=n_max_proxies,
            )

            # Update state to get correct total_request_reward_amount
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
            stake_amount=delegation_state_response.total_request_reward_amount,
        )
