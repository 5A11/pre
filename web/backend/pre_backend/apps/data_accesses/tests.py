"""Tests of the DataAccess app."""

from django.contrib.auth import get_user_model
from django.core.files.uploadedfile import SimpleUploadedFile
from django.test import TestCase
from django.urls import reverse
from unittest.mock import patch

from pre.api.delegatee import DelegateeAPI
from pre.api.delegator import DelegatorAPI
from pre.contract.cosmos_contracts import ContractQueries
from pre.ledger.cosmos.ledger import BroadcastException

from .constants import DATA_ID_MAX_LENGTH
from .helpers import DelegateeSDK, DelegatorSDK
from .models import DataAccess


User = get_user_model()


# Status codes
# This is repeated in test modules of different apps
# TODO: find a better place to keep that and import
HTTP_OK = 200
HTTP_CREATED = 201
HTTP_BAD_REQUEST = 400
HTTP_UNAUTHORIZED = 401


class DataAccessListAPIViewTestCase(TestCase):
    """Test case for DataAccessListAPIView."""

    fixtures = [
        "pre_backend/fixtures/data_accesses.json",
        "pre_backend/fixtures/users.json",
    ]

    def test_unauthorized_access(self):
        """Test for unauthorized access."""
        url = reverse("data-access-list")
        response = self.client.get(url)
        self.assertEqual(response.status_code, HTTP_UNAUTHORIZED)

    def test_get_data_access_list_positive(self):
        """Test for DataAccess list positive result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("data-access-list")
        response = self.client.get(url)
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_OK)
        expected_result = [
            {
                "id": 1,
                "owner": 1,
                "readers": [2],  # Represented with PK
            },
            {
                "id": 2,
                "owner": 2,
                "readers": [1],
            },
        ]
        self.assertEqual(result, expected_result)


class DataAccessAPIViewTestCase(TestCase):
    """Test case for DataAccessAPIView."""

    fixtures = [
        "pre_backend/fixtures/data_accesses.json",
        "pre_backend/fixtures/users.json",
    ]

    def test_unauthorized_access(self):
        """Test for unauthorized access."""
        pk = 1
        url = reverse("data-access", kwargs={"pk": pk})
        response = self.client.get(url)
        self.assertEqual(response.status_code, HTTP_UNAUTHORIZED)

    def test_get_data_access_positive(self):
        """Test for DataAccess retrieve positive result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        pk = 1
        url = reverse("data-access", kwargs={"pk": pk})

        response = self.client.get(url)
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_OK)

        expected_result = {
            "id": 1,
            "owner": 1,
            "readers": [2],
        }
        self.assertEqual(result, expected_result)


class DataAccessCreateTestCase(TestCase):
    """Test case for DataAccessCreateAPIView."""

    fixtures = [
        "pre_backend/fixtures/users.json",
        "pre_backend/fixtures/user_profiles.json",
    ]

    def test_unauthorized_access(self):
        """Test for unauthorized access."""
        url = reverse("data-access-create")
        response = self.client.post(url, {})
        self.assertEqual(response.status_code, HTTP_UNAUTHORIZED)

    @patch(
        "pre_backend.apps.data_accesses.serializers.DelegatorSDK.add_data",
        return_value="d" * DATA_ID_MAX_LENGTH,
    )
    def test_data_access_create_positive(self, *sdk_add_data_mock):
        """Test for DataAccess create positive result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        filename = "some.file"
        testfile = SimpleUploadedFile(filename, b"content")

        url = reverse("data-access-create")

        response = self.client.post(
            url,
            {"file": testfile},
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_CREATED)
        expected_result = {}
        self.assertEqual(result, expected_result)

    def test_data_access_create_bad_data(self):
        """Test for DataAccess create negative result: bad data provided."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("data-access-create")

        response = self.client.post(
            url,
            {
                "file": "some-string",
            },
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        expected_result = {
            "file": [
                "The submitted data was not a file. Check the encoding type on the form."
            ]
        }
        self.assertEqual(result, expected_result)

    def test_data_access_create_negative_missing_data(self):
        """Test for DataAccess create negative result: missing data."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("data-access-create")
        response = self.client.post(
            url,
            {},  # Empty data
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        expected_result = {"file": ["No file was submitted."]}
        self.assertEqual(result, expected_result)


class DelegatorSDKTestCase(TestCase):
    """Test case for DelegatorSDK class."""

    fixtures = [
        "pre_backend/fixtures/users.json",
        "pre_backend/fixtures/user_profiles.json",
    ]

    def test__get_delegator_api_positive(self):
        """Test _get_delegator_api staticmethod for positive result."""
        username = "admin"
        user = User.objects.get(username=username)
        delegator_api = DelegatorSDK._get_delegator_api(user)
        self.assertIsInstance(delegator_api, DelegatorAPI)

    def test_add_data_positive(self):
        """Test add_data method for positive result."""
        username = "admin"
        user = User.objects.get(username=username)
        delegator_sdk = DelegatorSDK(user)

        # TODO: update next check when issue with contract is solved.
        with self.assertRaises(BroadcastException) as e:
            delegator_sdk.add_data(b"data")
        self.assertIn(
            "Getting account data failed after multiple attempts", str(e.exception)
        )
        # expected_result = "correct-hash-id"
        # self.assertEqual(result, expected_result)

    def test_grant_access_positive(self):
        """Test grant_access method for positive result."""
        owner_username = "admin"
        reader_username = "user"
        owner = User.objects.get(username=owner_username)
        reader = User.objects.get(username=reader_username)

        data_id = "z" * DATA_ID_MAX_LENGTH
        new_data_access = DataAccess.objects.create(data_id=data_id, owner=owner)

        delegator_sdk = DelegatorSDK(owner)

        # TODO: update next check when issue with contract is solved.
        with self.assertRaises(BroadcastException) as e:
            delegator_sdk.grant_access(new_data_access, reader)
        self.assertIn(
            "Getting contract state failed after multiple attempts", str(e.exception)
        )


class DelegateeSDKTestCase(TestCase):
    """Test case for DelegateeSDK class."""

    fixtures = [
        "pre_backend/fixtures/users.json",
        "pre_backend/fixtures/user_profiles.json",
    ]

    def test__get_delegatee_api_and_query_contract_positive(self):
        """Test _get_delegatee_api_and_query_contract method for positive result."""
        username = "admin"
        user = User.objects.get(username=username)
        (
            delegatee_api,
            query_contract,
        ) = DelegateeSDK._get_delegatee_api_and_query_contract(user)
        self.assertIsInstance(delegatee_api, DelegateeAPI)
        self.assertIsInstance(query_contract, ContractQueries)

    def test_get_data_positive(self):
        """Test get_data method for positive result."""
        owner_username = "admin"
        reader_username = "user"
        owner = User.objects.get(username=owner_username)
        reader = User.objects.get(username=reader_username)

        data_id = "z" * DATA_ID_MAX_LENGTH
        new_data_access = DataAccess.objects.create(data_id=data_id, owner=owner)
        new_data_access.readers.add(reader)
        new_data_access.save()

        delegatee_sdk = DelegateeSDK(reader)
        with self.assertRaises(BroadcastException) as e:
            delegatee_sdk.get_data(new_data_access)
        self.assertIn(
            "Getting contract state failed after multiple attempts", str(e.exception)
        )
