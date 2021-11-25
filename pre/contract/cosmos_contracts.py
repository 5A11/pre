from base64 import b64decode, b64encode
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from pre.common import Address, Delegation, HashID, ProxyTask
from pre.contract.base_contract import (
    AbstractAdminContract,
    AbstractContractQueries,
    AbstractDelegatorContract,
    AbstractProxyContract,
)
from pre.ledger.base_ledger import AbstractLedgerCrypto
from pre.ledger.cosmos.ledger import CosmosLedger
from pre.utils.loggers import get_logger


CONTRACT_WASM_FILE = str(Path(__file__).parent.parent.parent / "contract" / "./cw_proxy_reencryption.wasm")

_logger = get_logger(__name__)


def encode_bytes(data: bytes) -> str:
    return b64encode(data).decode("ascii")


class ContractQueries(AbstractContractQueries):
    ledger: CosmosLedger

    def get_avaiable_proxies(self) -> List[bytes]:
        state_msg: Dict = {"get_available_proxies": {}}
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return [b64decode(i) for i in json_res["proxy_pubkeys"]]

    def get_selected_proxies_for_delegation(
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        state_msg: Dict = {
            "get_selected_proxies_for_delegation": {
                "delegator_addr": delegator_addr,
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return [b64decode(i) for i in json_res["proxy_pubkeys"]]

    def get_threshold(self) -> int:
        state_msg: Dict = {"get_threshold": {}}
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return json_res["threshold"]

    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        state_msg: Dict = {
            "get_next_proxy_task": {"proxy_pubkey": encode_bytes(proxy_pubkey_bytes)}
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)

        if json_res["proxy_task"]:
            return ProxyTask(
                hash_id=json_res["proxy_task"]["data_id"],
                delegatee_pubkey=b64decode(json_res["proxy_task"]["delegatee_pubkey"]),
                delegator_pubkey=b64decode(json_res["proxy_task"]["delegator_pubkey"]),
                delegation_string=b64decode(
                    json_res["proxy_task"]["delegation_string"]
                ),
            )
        return None

    def get_fragments_response(
        self, hash_id: HashID, delegatee_pubkey_bytes: bytes
    ) -> Tuple[int, List[HashID]]:
        state_msg: Dict = {
            "get_fragments": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return json_res["threshold"], json_res["fragments"]

    def does_delegation_exist(
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bool:
        state_msg: Dict = {
            "get_does_delegation_exist": {
                "delegator_addr": delegator_addr,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
            }
        }
        json_res = self.ledger.send_query_msg(self.contract_address, state_msg)
        return json_res["delegation_exists"]


class AdminContract(AbstractAdminContract):
    CONTRACT_WASM_FILE = CONTRACT_WASM_FILE
    ledger: CosmosLedger

    @classmethod
    def instantiate_contract(
        cls,
        ledger: CosmosLedger,
        admin_private_key: AbstractLedgerCrypto,
        admin_addr: Address,
        threshold: Optional[int],
        n_max_proxies: Optional[int],
        proxies: Optional[List[Address]] = None,
        label: str = "PRE",
    ) -> Address:
        _logger.info("Deploying contract")
        code_id, res = ledger.deploy_contract(admin_private_key, cls.CONTRACT_WASM_FILE)

        if not isinstance(code_id, int):
            raise ValueError(f"Bad response on deploy contract: {code_id} {res}")

        _logger.info("Initializing deployed contract")
        init_msg: Dict = {
            "admin": admin_addr,
            "proxies": proxies or [],
        }
        if threshold is not None:
            init_msg["threshold"] = threshold

        if n_max_proxies is not None:
            init_msg["n_max_proxies"] = n_max_proxies

        contract_address, res = ledger.send_init_msg(
            admin_private_key, code_id, init_msg, label
        )
        return Address(contract_address)

    def add_proxy(self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address):
        submit_msg = {"add_proxy": {"proxy_addr": proxy_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

    def remove_proxy(
        self, admin_private_key: AbstractLedgerCrypto, proxy_addr: Address
    ):
        submit_msg = {"remove_proxy": {"proxy_addr": proxy_addr}}
        res, error_code = self.ledger.send_execute_msg(
            admin_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")


class DelegatorContract(AbstractDelegatorContract):
    ledger: CosmosLedger

    def add_data(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
    ):
        submit_msg = {
            "add_data": {
                "data_id": str(hash_id),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

    def add_delegations(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
        delegations: List[Delegation],
    ):
        submit_msg = {
            "add_delegation": {
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "proxy_delegations": [
                    {
                        "proxy_pubkey": encode_bytes(i.proxy_pub_key),
                        "delegation_string": encode_bytes(i.delegation_string),
                    }
                    for i in delegations
                ],
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

    def does_delegation_exist(
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> bool:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).does_delegation_exist(
            delegator_addr, delegator_pubkey_bytes, delegatee_pubkey_bytes
        )

    def get_selected_proxies_for_delegation(
        self,
        delegator_addr: Address,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_selected_proxies_for_delegation(
            delegator_addr, delegator_pubkey_bytes, delegatee_pubkey_bytes
        )

    def request_proxies_for_delegation(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        delegatee_pubkey_bytes: bytes,
    ) -> List[bytes]:
        submit_msg = {
            "request_proxies_for_delegation": {
                "delegator_pubkey": encode_bytes(delegator_pubkey_bytes),
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

        # FIXME(LR) parse `res`` instead
        return self.get_selected_proxies_for_delegation(
            delegator_private_key.get_address(),
            delegator_pubkey_bytes,
            delegatee_pubkey_bytes,
        )

    def request_reencryption(
        self,
        delegator_private_key: AbstractLedgerCrypto,
        delegator_pubkey_bytes: bytes,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
    ):
        submit_msg = {
            "request_reencryption": {
                "delegator_public_key": encode_bytes(delegator_pubkey_bytes),
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "data_id": str(hash_id),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            delegator_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

    def get_avaiable_proxies(self) -> List[bytes]:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_avaiable_proxies()


class ProxyContract(AbstractProxyContract):
    ledger: CosmosLedger

    def proxy_register(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        proxy_pubkey_bytes: bytes,
    ):
        submit_msg = {
            "register_proxy": {
                "proxy_pubkey": encode_bytes(proxy_pubkey_bytes),
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

    def proxy_unregister(
        self,
        proxy_private_key: AbstractLedgerCrypto,
    ):
        submit_msg: Dict = {"unregister_proxy": {}}
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")

    def get_next_proxy_task(self, proxy_pubkey_bytes: bytes) -> Optional[ProxyTask]:
        return ContractQueries(
            ledger=self.ledger, contract_address=self.contract_address
        ).get_next_proxy_task(proxy_pubkey_bytes)

    def provide_reencrypted_fragment(
        self,
        proxy_private_key: AbstractLedgerCrypto,
        hash_id: HashID,
        delegatee_pubkey_bytes: bytes,
        fragment_hash_id: HashID,
    ):
        submit_msg = {
            "provide_reencrypted_fragment": {
                "data_id": hash_id,
                "delegatee_pubkey": encode_bytes(delegatee_pubkey_bytes),
                "fragment": fragment_hash_id,
            }
        }
        res, error_code = self.ledger.send_execute_msg(
            proxy_private_key, self.contract_address, submit_msg
        )
        if error_code != 0:
            raise ValueError(f"Contract execution failed: {error_code} {res}")
