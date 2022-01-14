import pytest

from pre.common import EncryptedData
from pre.storage.base_storage import (
    StorageError,
    StorageNotAvailable,
    StorageNotConnected,
    StorageTimeout,
)
from pre.storage.ipfs_storage import IpfsStorage

from tests.utils import local_ledger_and_storage


def test_storage_errors():
    with local_ledger_and_storage() as confs:
        _, ipfs_conf = confs

        storage = IpfsStorage(**ipfs_conf)

        with pytest.raises(StorageError, match="is not connected"):
            storage.disconnect()

        with pytest.raises(StorageNotAvailable, match="is not avaiable with address:"):
            storage = IpfsStorage(**{**ipfs_conf, "port": 65001})
            storage.connect()

        storage = IpfsStorage(**ipfs_conf)
        storage.connect()

        with pytest.raises(StorageError, match="Already connected!"):
            storage.connect()

        enc_data = EncryptedData(b"123", b"345")
        hash_id = storage.store_encrypted_data(enc_data)
        stored_enc_data = storage.get_encrypted_data(hash_id, stream=False)
        assert enc_data == stored_enc_data

        assert enc_data.data == storage.get_data(hash_id, stream=False)
        assert enc_data.capsule == storage.get_capsule(hash_id)

        with pytest.raises(StorageTimeout):
            storage.get_data(
                hash_id="Qme7ss3ARVgxv6rXqVPiikMJ8u2NLgmgszg13pYrDKEoia", stream=False
            )

        storage.disconnect()
        with pytest.raises(
            StorageNotConnected, match=r"storage is not connected! Connect first!"
        ):
            storage.disconnect()

        # do not allow to run without proper options
        with pytest.raises(
            ValueError, match="Provide `addr` value in storage_config or addr and port"
        ):
            IpfsStorage()


def test_storage_config():
    IpfsStorage.CONFIG_CLASS.validate(IpfsStorage.CONFIG_CLASS.make_default())

    # ok use defaults
    IpfsStorage.CONFIG_CLASS.validate({})

    with pytest.raises(
        ValueError, match="ipfs storage parameter `addr` has to be a string"
    ):
        IpfsStorage.CONFIG_CLASS.validate({"addr": 32})

    with pytest.raises(
        ValueError, match="ipfs storage parameter `port` has to be an integer"
    ):
        IpfsStorage.CONFIG_CLASS.validate({"port": "str"})

    with pytest.raises(
        ValueError, match="ipfs storage parameter `timeout` has to be an integer"
    ):
        IpfsStorage.CONFIG_CLASS.validate({"timeout": "sdf"})
