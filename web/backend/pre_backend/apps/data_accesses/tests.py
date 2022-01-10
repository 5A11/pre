"""Tests of the DataAccess app."""

from django.test import TestCase
from django.urls import reverse

from .constants import DATA_ID_MAX_LENGTH

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
                "data_id": "a" * DATA_ID_MAX_LENGTH,
                "owner": "admin",
                "readers": [2],  # Represented with PK
            },
            {
                "id": 2,
                "data_id": "b" * DATA_ID_MAX_LENGTH,
                "owner": "user",
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
            "data_id": "a" * DATA_ID_MAX_LENGTH,
            "owner": "admin",
            "readers": [2],
        }
        self.assertEqual(result, expected_result)


class DataAccessCreateTestCase(TestCase):
    """Test case for DataAccessCreateAPIView."""

    fixtures = [
        "pre_backend/fixtures/users.json",
    ]

    def test_unauthorized_access(self):
        """Test for unauthorized access."""
        url = reverse("data-access-create")
        response = self.client.post(url, {})
        self.assertEqual(response.status_code, HTTP_UNAUTHORIZED)

    def test_data_access_create_positive(self):
        """Test for DataAccess create positive result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("data-access-create")

        data_id = "a" * DATA_ID_MAX_LENGTH
        readers = [2]  # PK (ID) of user with username "user"

        response = self.client.post(
            url,
            {
                "data_id": data_id,
                "readers": readers,
            },
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_CREATED)
        expected_result = {
            "id": 3,
            "data_id": data_id,
            "owner": "admin",
            "readers": readers,
        }
        self.assertEqual(result, expected_result)

    def test_data_access_create_bad_data(self):
        """Test for DataAccess create negative result: bad data provided."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("data-access-create")

        data_id = "a" * (DATA_ID_MAX_LENGTH + 1)  # Too long string
        readers = ["abc"]  # Incorrect type

        response = self.client.post(
            url,
            {
                "data_id": data_id,
                "readers": readers,
            },
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        expected_result = {
            "data_id": [
                f"Ensure this field has no more than {DATA_ID_MAX_LENGTH} characters."
            ],
            "readers": ["Incorrect type. Expected pk value, received str."],
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
        expected_result = {
            "data_id": ["This field is required."],
            "readers": ["This list may not be empty."],
        }
        self.assertEqual(result, expected_result)
