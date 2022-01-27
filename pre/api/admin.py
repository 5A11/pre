from typing import List, Optional, Type

from pre.common import Address
from pre.contract.base_contract import AbstractAdminContract
from pre.contract.cosmos_contracts import AdminContract
from pre.ledger.base_ledger import AbstractLedger, AbstractLedgerCrypto


class AdminAPI:
    """Admin API class to set contract and perform contract administration."""

    _contract: AbstractAdminContract
    CONTRACT_CLASS = AdminContract

    def __init__(
        self,
        ledger_crypto: AbstractLedgerCrypto,
        contract: AbstractAdminContract,
    ):
        """
        Init AdminAPI instance.

        :param ledger_crypto: ledger private key instance to perform ledger operations
        :param contract: AdminContract instance.
        """

        self._ledger_crypto = ledger_crypto
        self._contract = contract

    @classmethod
    def instantiate_contract(
        cls,
        ledger_crypto: AbstractLedgerCrypto,
        ledger: AbstractLedger,
        admin_address: Address,
        stake_denom: str,
        minimum_proxy_stake_amount: Optional[str] = None,
        per_proxy_request_reward_amount: Optional[str] = None,
        per_request_slash_stake_amount: Optional[str] = None,
        threshold: Optional[int] = None,
        n_max_proxies: Optional[int] = None,
        proxies: Optional[List[Address]] = None,
        label: str = "PRE",
        contract_cls: Optional[Type[AbstractAdminContract]] = None,
    ) -> Address:
        """
        Instantiate contract.

        :param ledger_crypto: private ledger key instance
        :param ledger: ledger instance to perform contract deployment
        :param admin_address: address of contract administator
        :param stake_denom: str,
        :param minimum_proxy_stake_amount: Optional[str],
        :param per_proxy_request_reward_amount: Optional[str] = None,
        :param threshold: int threshold ,
        :param n_max_proxies: max amount of proxy allowed to register,
        :param proxies: optional list of proxies addresses,
        :param label: str, contract label
        :param contract_cls: Optional[Type[AbstractAdminContract]] = None,
        """

        contract_cls = contract_cls or cls.CONTRACT_CLASS
        contract_address = contract_cls.instantiate_contract(
            ledger,
            ledger_crypto,
            admin_address,
            stake_denom,
            minimum_proxy_stake_amount,
            per_proxy_request_reward_amount,
            per_request_slash_stake_amount,
            threshold,
            n_max_proxies,
            proxies or [],
            label,
        )
        return contract_address

    def add_proxy(self, proxy_address: Address):
        """
        Add proxy to allowed proxy list.

        :param proxy_addres: str
        :return: None
        """
        self._contract.add_proxy(self._ledger_crypto, proxy_address)

    def remove_proxy(self, proxy_address: Address):
        """
        Remove proxy from allowed proxy list.

        :param proxy_addres: str

        :return: None
        """

        self._contract.remove_proxy(self._ledger_crypto, proxy_address)
