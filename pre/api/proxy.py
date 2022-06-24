from typing import List, Optional

from pre.common import Coin, PrivateKey, ProxyState, ProxyStatus, ProxyTask
from pre.contract.base_contract import AbstractProxyContract
from pre.crypto.base_crypto import AbstractCrypto
from pre.ledger.base_ledger import AbstractLedgerCrypto


class ProxyAPI:
    """Proxy API to perform key reencryption by request from contact side."""

    def __init__(
        self,
        encryption_private_key: PrivateKey,
        ledger_crypto: AbstractLedgerCrypto,
        contract: AbstractProxyContract,
        crypto: AbstractCrypto,
    ):
        """
        Init api isntance.

        :param encryption_private_key: PrivateKey,
        :param ledger_crypto: ledger private key instance,
        :param contract: instance of proxy contract implementation,
        :param crypto: instance of abstract crypto implementation,
        """
        self._encryption_private_key = encryption_private_key
        self._ledger_crypto = ledger_crypto
        self._contract = contract
        self._crypto = crypto

    def _pub_key_as_bytes(self) -> bytes:
        """Get proxy crypto public key in bytes."""
        return bytes(self._encryption_private_key.public_key)

    def registered(self) -> bool:
        status = self._contract.get_proxy_status(self._ledger_crypto.get_address())
        return (
            status.proxy_state == ProxyState.registered if status is not None else False
        )

    def register(self) -> Optional[Coin]:
        """Register a proxy on the specific contract."""
        status = self._contract.get_proxy_status(self._ledger_crypto.get_address())
        state = status.proxy_state if status is not None else ProxyState.authorised
        if state == ProxyState.registered:
            return None

        if state == ProxyState.leaving:
            minimum_registration_stake = None
        else:
            staking_config = self._contract.get_staking_config()
            minimum_registration_stake = Coin(
                denom=staking_config.stake_denom,
                amount=str(staking_config.minimum_proxy_stake_amount),
            )

        self._contract.proxy_register(
            proxy_private_key=self._ledger_crypto,
            proxy_pubkey_bytes=self._pub_key_as_bytes(),
            stake_amount=minimum_registration_stake,
        )

        return minimum_registration_stake

    def unregister(self, deactivate_only: bool = False):
        """Unregister proxy."""
        self._contract.proxy_deactivate(self._ledger_crypto)
        if not deactivate_only:
            self._contract.proxy_unregister(self._ledger_crypto)

    def deactivate(self):
        """Deactivate proxy."""
        self._contract.proxy_deactivate(self._ledger_crypto)

    def reactivate(self):
        """Activate proxy after being deactivated."""
        self._contract.proxy_register(
            proxy_private_key=self._ledger_crypto,
            proxy_pubkey_bytes=self._pub_key_as_bytes(),
        )

    def withdraw_stake(self, stake_amount: Optional[int] = None):
        """Withdraw proxy stake."""
        self._contract.withdraw_stake(self._ledger_crypto, stake_amount)

    def get_reencryption_requests(
        self,
    ) -> List[ProxyTask]:
        """
        Get reencryption tasks from the contract.

        :return: List of ProxyTask
        """
        return self._contract.get_proxy_tasks(self._ledger_crypto.get_address())

    def skip_task(self, task: ProxyTask):
        """Skip task for processing"""

        self._contract.skip_reencryption_task(
            self._ledger_crypto, task.hash_id, self._pub_key_as_bytes()
        )

    def process_reencryption_request(self, proxy_task: ProxyTask):
        """
        Process reencryption request.
        Make reencrypted fragment, store it in contract, register its hash_id in the contract.

        :param proxy_task: ProxyTask instance from get_reencryption_requests
        """
        hash_id = proxy_task.hash_id
        delegatee_pubkey_bytes = proxy_task.delegatee_pubkey
        capsule = proxy_task.capsule
        encrypted_fragment = self._crypto.reencrypt(
            capsule_bytes=capsule,
            delegation_bytes=proxy_task.delegation_string,
            proxy_private_key=self._encryption_private_key,
            delegator_pubkey_bytes=proxy_task.delegator_pubkey,
            delegatee_pubkey_bytes=proxy_task.delegatee_pubkey,
        )
        self._contract.provide_reencrypted_fragment(
            proxy_private_key=self._ledger_crypto,
            hash_id=hash_id,
            delegatee_pubkey_bytes=delegatee_pubkey_bytes,
            fragment_bytes=encrypted_fragment,
        )

    def get_proxy_status(
        self,
    ) -> Optional[ProxyStatus]:
        """
        Get proxy status.

        :return: None or ProxyStatus instance
        """
        return self._contract.get_proxy_status(self._ledger_crypto.get_address())
