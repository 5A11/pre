"""Tests of Authorization app."""

from django.contrib.auth import get_user_model
from django.test import TestCase
from django.urls import reverse

from .constants import ENCRYPTION_MAX_LENGTH, LEDGER_MAX_LENGTH
from .models import UserProfile


User = get_user_model()

# Status codes
HTTP_OK = 200
HTTP_CREATED = 201
HTTP_BAD_REQUEST = 400
HTTP_NOT_FOUND = 404

ENCRYPTION_EXAMPLE = "a" * ENCRYPTION_MAX_LENGTH
LEDGER_EXAMPLE = "b" * LEDGER_MAX_LENGTH


class RegistrationTestCase(TestCase):
    """Test case for Registration endpoint."""

    def test_registration_positive(self):
        """Test registration endpoint for positive result."""
        url = reverse("rest_register")
        username = "user"
        data = {
            "username": username,
            "email": "email@example.com",
            "password1": "p@ssw00rd",
            "password2": "p@ssw00rd",
        }
        response = self.client.post(url, data)
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_CREATED)

        # If email verification is optional
        self.assertTrue("key" in result.keys())

        # If email verification is mandatory
        # expected_result = {"detail": "Verification e-mail sent."}
        # self.assertEqual(result, expected_result)

        queryset = User.objects.filter(username=username)
        self.assertTrue(queryset.exists())

        # Check UserProfile is created
        obj = queryset.first()
        self.assertTrue(hasattr(obj, "userprofile"))

    def test_registration_negative_invalid_data(self):
        """Test registration endpoint for negative result: invalid data provided."""
        url = reverse("rest_register")
        username = "user"
        data = {
            "username": username,
            "email": "email",  # Invalid email
            "password1": "111",  # Bad password
            "password2": "111",
        }
        response = self.client.post(url, data)
        result, status_code = response.json(), response.status_code
        expected_result = {
            "email": ["Enter a valid email address."],
            "password1": [
                "This password is too short. It must contain at least 8 characters.",
                "This password is too common.",
                "This password is entirely numeric.",
            ],
        }
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        self.assertEqual(result, expected_result)
        self.assertFalse(User.objects.filter(username=username).exists())

    def test_registration_negative_passwords_didnt_match(self):
        """Test registration endpoint for negative result: passwords didn't match."""
        url = reverse("rest_register")
        username = "user"
        data = {
            "username": username,
            "email": "email@example.com",
            "password1": "p@ssw00rd",
            "password2": "passwoord",
        }
        response = self.client.post(url, data)
        result, status_code = response.json(), response.status_code
        expected_result = {
            "non_field_errors": ["The two password fields didn't match."]
        }
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        self.assertEqual(result, expected_result)
        self.assertFalse(User.objects.filter(username=username).exists())


class RegistrationTestCaseWithFixtures(TestCase):
    """Test case for Registration endpoint with fixtures."""

    fixtures = ["pre_backend/fixtures/users.json"]

    def test_registration_negative_user_exists(self):
        """Test registration endpoint for negative result: user exists."""
        url = reverse("rest_register")
        username = "user"  # Already present in fixture
        data = {
            "username": username,
            "email": "email@example.com",
            "password1": "p@ssw00rd",
            "password2": "p@ssw00rd",
        }
        response = self.client.post(url, data)
        result, status_code = response.json(), response.status_code
        expected_result = {"username": ["A user with that username already exists."]}
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        self.assertEqual(result, expected_result)


class LoginTestCase(TestCase):
    """Test case for Login endpoint."""

    def test_login_positive(self):
        """Test login endpoint for positive result."""
        url = reverse("rest_login")
        username = "user"
        email = "email@example.com"
        password = "p@ssw00rd"

        User.objects.create_user(username=username, email=email, password=password)
        data = {
            "username": username,
            "password": password,
        }
        response = self.client.post(url, data)
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_OK)
        self.assertTrue("key" in result.keys())

    def test_login_negative(self):
        """Test login endpoint for negative result."""
        url = reverse("rest_login")
        username = "user"
        password = "p@ssw00rd"
        data = {
            "username": username,
            "password": password,
        }
        response = self.client.post(url, data)
        result, status_code = response.json(), response.status_code
        expected_result = {
            "non_field_errors": ["Unable to log in with provided credentials."]
        }
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        self.assertEqual(result, expected_result)


class UserProfileAPIViewTestCase(TestCase):
    """Test case for UserProfileAPIView."""

    fixtures = [
        "pre_backend/fixtures/users.json",
        "pre_backend/fixtures/user_profiles.json",
    ]

    def test_get_user_profile_positive(self):
        """Test Get user profile for positive result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        user_profile = UserProfile.objects.get(user__username=username)

        url = reverse("user-profile")
        response = self.client.get(url)
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_OK)
        expected_result = {
            "encryption": user_profile.encryption,
            "ledger": user_profile.ledger,
        }
        self.assertEqual(result, expected_result)

    def test_get_user_profile_negative(self):
        """Test Get user profile for negative result: user does not have UserProfile."""
        username = "new_user"
        password = "p@ssw0rd"
        email = "email@example.com"
        User.objects.create_user(username, email, password)

        self.client.login(username=username, password=password)

        url = reverse("user-profile")
        response = self.client.get(url)
        self.assertEqual(response.status_code, HTTP_NOT_FOUND)

    def test_update_user_profile_positive(self):
        """Test update UseProfile for positive result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("user-profile")
        data = {
            "encryption": "e" * ENCRYPTION_MAX_LENGTH,
            "ledger": "f" * LEDGER_MAX_LENGTH,
        }
        response = self.client.put(
            url,
            data,
            content_type="application/json",
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_OK)
        expected_result = data
        self.assertEqual(result, expected_result)

    def test_update_user_profile_negative(self):
        """Test update UseProfile for negative result."""
        username = "admin"
        self.client.login(username=username, password="admin")

        url = reverse("user-profile")
        data = {
            "encryption": "e" * (ENCRYPTION_MAX_LENGTH + 1),
            "ledger": "f" * (LEDGER_MAX_LENGTH + 1),
        }
        response = self.client.put(
            url,
            data,
            content_type="application/json",
        )
        result, status_code = response.json(), response.status_code
        self.assertEqual(status_code, HTTP_BAD_REQUEST)
        expected_result = {
            "encryption": [
                f"Ensure this field has no more than {ENCRYPTION_MAX_LENGTH} characters."
            ],
            "ledger": [
                f"Ensure this field has no more than {LEDGER_MAX_LENGTH} characters."
            ],
        }
        self.assertEqual(result, expected_result)
