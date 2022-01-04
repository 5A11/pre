from typing import List, Optional, Type

from pre.common import Address
from pre.contract.base_contract import AbstractAdminContract
from pre.contract.cosmos_contracts import AdminContract
from pre.ledger.base_ledger import AbstractLedger, AbstractLedgerCrypto


class AdminAPI:
    _contract: AbstractAdminContract
    CONTRACT_CLASS = AdminContract

    def __init__(
        self,
        ledger_crypto: AbstractLedgerCrypto,
        contract: AbstractAdminContract,
    ):
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
        minimum_request_reward_amount: Optional[str] = None,
        per_request_slash_stake_amount: Optional[str] = None,
        threshold: Optional[int] = None,
        n_max_proxies: Optional[int] = None,
        proxies: Optional[List[Address]] = None,
        label: str = "PRE",
        contract_cls: Optional[Type[AbstractAdminContract]] = None,
    ) -> Address:
        contract_cls = contract_cls or cls.CONTRACT_CLASS
        contract_address = contract_cls.instantiate_contract(
            ledger,
            ledger_crypto,
            admin_address,
            stake_denom,
            minimum_proxy_stake_amount,
            minimum_request_reward_amount,
            per_request_slash_stake_amount,
            threshold,
            n_max_proxies,
            proxies or [],
            label,
        )
        return contract_address

    def add_proxy(self, proxy_address: Address):
        self._contract.add_proxy(self._ledger_crypto, proxy_address)

    def remove_proxy(self, proxy_address: Address):
        self._contract.remove_proxy(self._ledger_crypto, proxy_address)
