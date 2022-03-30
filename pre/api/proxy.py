from typing import Optional

from cosmpy.protos.cosmos.base.v1beta1.coin_pb2 import Coin

from pre.common import PrivateKey, ProxyTask
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

    def register(self):
        """Register a proxy on the specific contract."""
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

    def unregister(self):
        """Unregister proxy."""
        self._contract.proxy_unregister(self._ledger_crypto)

    def withdraw_stake(self, stake_amount: Optional[int] = None):
        """Withdraw proxy stake."""
        self._contract.withdraw_stake(self._ledger_crypto, stake_amount)

    def get_next_reencryption_request(
        self,
    ) -> Optional[ProxyTask]:
        """
        Get next reencryption task from the contract.

        :return: ProxyTask or None
        """
        return self._contract.get_next_proxy_task(self._pub_key_as_bytes())

    def process_reencryption_request(self, proxy_task: ProxyTask):
        """
        Process reencryption request.
        Make reencrypted fragment, store it in contract, register its hash_id in the contract.

        :param proxy_task: ProxyTask instance from get_next_reencryption_request
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
