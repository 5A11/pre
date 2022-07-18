import json
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional, Sequence, Union

import certifi
import click
import grpc
import requests
from admin import DeployedContract, LedgerNetworkConfig
from cosmpy.protos.cosmos.base.abci.v1beta1.abci_pb2 import TxResponse
from cosmpy.protos.cosmos.base.query.v1beta1.pagination_pb2 import PageRequest
from cosmpy.protos.cosmos.tx.v1beta1.service_pb2_grpc import ServiceStub as TxGrpcClient
from cosmpy.tx.rest_client import GetTxsEventRequest


PRINT_DELIM = "\t"


class EventType(Enum):
    add_data = "add_data"
    request_reencryption = "request_reencryption"
    ignored = "__ignored__"


@dataclass
class Event:
    tx_height: int
    tx_hash: str
    tx_timestamp: str
    attributes: Dict[str, Any]
    event_type: EventType = field(default=EventType.ignored, init=False)

    def __post_init__(self):
        pass

    def __str__(self) -> str:
        delim = PRINT_DELIM
        return delim.join(
            [
                str(self.tx_height),
                self.tx_timestamp,
                self.attributes["action"],
                self.tx_hash,
            ]
        )


@dataclass
class AddDataEvent(Event):
    owner: str
    data_id: str

    def __post_init__(self):
        super().__post_init__()
        self.event_type = EventType.add_data

    def __str__(self) -> str:
        delim = PRINT_DELIM
        return delim.join(
            [
                str(self.tx_height),
                self.tx_timestamp,
                self.owner,
                self.data_id,
                self.tx_hash,
            ]
        )


@dataclass
class RequestReencryptionEvent(Event):
    data_id: str
    reader: str

    def __post_init__(self):
        super().__post_init__()
        self.event_type = EventType.request_reencryption

    def __str__(self) -> str:
        delim = PRINT_DELIM
        return delim.join(
            [
                str(self.tx_height),
                self.tx_timestamp,
                self.data_id,
                self.reader,
                self.tx_hash,
            ]
        )


class EventQuerier:
    def __init__(self, contract: DeployedContract) -> None:
        self.addr = contract.contract_address
        with open(certifi.where(), "rb") as f:
            trusted_certs = f.read()
        credentials = grpc.ssl_channel_credentials(root_certificates=trusted_certs)
        channel = grpc.secure_channel(contract.network.node_address, credentials)
        self.tx = TxGrpcClient(channel)

    def get(self, event_type: Optional[EventType] = None) -> List[Event]:
        queries = [
            f"execute._contract_address='{self.addr}'",
            # f"wasm.contract_address='{self.addr}'",
        ]

        if event_type is not None and event_type != EventType.ignored:
            queries.append(f"wasm.action='{event_type.value}'")

        return self._query(queries, event_type)

    def _query(
        self, queries: List[str], event_type: Optional[EventType] = None
    ) -> List[Event]:
        offset = 0
        limit = 100
        events_all = []

        while True:
            res = self.tx.GetTxsEvent(
                GetTxsEventRequest(
                    events=queries, pagination=PageRequest(offset=offset, limit=limit)
                )
            )

            events = self._parse_events_response(res.tx_responses)
            if event_type is None:
                events_all.extend(events)
            else:
                events_all.extend([e for e in events if e.event_type == event_type])

            offset += len(res.tx_responses)
            if offset >= res.pagination.total:
                break

        return events_all

    def _parse_events_response(self, events: List[TxResponse]) -> List[Event]:
        parsed = []
        for e in events:
            kwargs = {
                "tx_height": e.height,
                "tx_timestamp": e.timestamp,
                "tx_hash": e.txhash,
            }

            for te in e.events:
                if te.type == "wasm":
                    attrs: Dict[str, Any] = {}
                    is_action = False
                    for attr in te.attributes:
                        if attr.key.decode("utf-8") == "action":
                            is_action = True
                        attrs[attr.key.decode("utf-8")] = attr.value.decode("utf-8")
                    if is_action:
                        kwargs["attributes"] = attrs
                        parsed.append(self._parse_event(kwargs))
        return parsed

    def _parse_event(self, kwargs: Dict[str, Any]) -> Event:
        attrs = kwargs["attributes"]
        if attrs["action"] == EventType.add_data.value:
            return AddDataEvent(
                owner=attrs["owner"],
                data_id=attrs["data_id"],
                **kwargs,
            )
        elif attrs["action"] == EventType.request_reencryption.value:
            return RequestReencryptionEvent(
                reader=attrs["delegatee_pubkey"],
                data_id=attrs["data_id"],
                **kwargs,
            )
        else:
            return Event(**kwargs)


def fetch_contract_info(url: str) -> DeployedContract:
    file = requests.get(url, allow_redirects=True)
    config = json.loads(file.content)
    return DeployedContract(
        contract_address=config["contract_address"],
        network=LedgerNetworkConfig(
            node_address=config["network"]["node_address"],
            chain_id=config["network"]["chain_id"],
            prefix=config["network"]["prefix"],
            denom=config["network"]["denom"],
        ),
    )


@click.group(name="query")
@click.argument("contract_url")
def cli(contract_url):
    pass


@cli.command()
@click.option("--data-registrations", "-d", is_flag=True)
@click.option("--reencryption-requests", "-r", is_flag=True)
@click.pass_context
def events(ctx: click.Context, data_registrations: bool, reencryption_requests: bool):
    contract = fetch_contract_info(ctx.parent.params["contract_url"])
    querier = EventQuerier(contract)

    events: Sequence[Union[str, Event]] = []

    if data_registrations:
        events = querier.get(EventType.add_data)
    elif reencryption_requests:
        events = querier.get(EventType.request_reencryption)
    else:
        events = querier.get()
        events = [Event.__str__(e) for e in events]  # hack

    print(*events, sep="\n")


if __name__ == "__main__":
    cli()
