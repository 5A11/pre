import logging
import os
import shutil
import signal
import subprocess
import time
from contextlib import contextmanager
from typing import Optional

import pytest

import docker as docker

from tests.constants import (
    DEFAULT_FETCH_DOCKER_IMAGE_TAG,
    DEFAULT_FETCH_LEDGER_ADDR,
    DEFAULT_FETCH_LEDGER_RPC_PORT,
)
from tests.docker_image import DockerImage, FetchLedgerDockerImage


logger = logging.getLogger(__name__)


def _launch_image(image: DockerImage, timeout: float = 2.0, max_attempts: int = 10):
    """
    Launch image.

    :param image: an instance of Docker image.
    :return: None
    """
    image.check_skip()
    image.stop_if_already_running()
    container = image.create()
    container.start()
    logger.info(f"Setting up image {image.tag}...")
    success = image.wait(max_attempts, timeout)
    if not success:
        container.stop()
        container.remove()
        pytest.fail(f"{image.tag} doesn't work. Exiting...")
    else:
        try:
            logger.info("Done!")
            time.sleep(timeout)
            yield
        finally:
            logger.info(f"Stopping the image {image.tag}...")
            # container.stop()
            # container.remove()


@contextmanager
def _fetchd_context(fetchd_configuration, timeout: float = 2.0, max_attempts: int = 20):
    client = docker.from_env()
    image = FetchLedgerDockerImage(
        client,
        DEFAULT_FETCH_LEDGER_ADDR,
        DEFAULT_FETCH_LEDGER_RPC_PORT,
        DEFAULT_FETCH_DOCKER_IMAGE_TAG,
        config=fetchd_configuration,
    )
    yield from _launch_image(image, timeout=timeout, max_attempts=max_attempts)


class IPFSDaemon:
    """
    Set up the IPFS daemon.

    :raises Exception: if IPFS is not installed.
    """

    def __init__(self, timeout: float = 15.0):
        """Initialise IPFS daemon."""
        # check we have ipfs
        self.timeout = timeout
        res = shutil.which("ipfs")
        if res is None:
            raise Exception("Please install IPFS first!")
        self.process = None  # type: Optional[subprocess.Popen]

    def __enter__(self) -> None:
        """Run the ipfs daemon."""
        self.process = subprocess.Popen(  # nosec
            ["ipfs", "daemon"],
            stdout=subprocess.PIPE,
            env=os.environ.copy(),
        )
        print("Waiting for {} seconds the IPFS daemon to be up.".format(self.timeout))
        time.sleep(self.timeout)

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:  # type: ignore
        """Terminate the ipfs daemon."""
        if self.process is None:
            return
        self.process.send_signal(signal.SIGTERM)
        self.process.wait(timeout=30)
        poll = self.process.poll()
        if poll is None:
            self.process.terminate()
            self.process.wait(2)
