from typing import IO, Union

from pre.api.base_api import BaseAPI
from pre.contract.base_contract import AbstractAdminContract


class Admin(BaseAPI):
    _contract: AbstractAdminContract
