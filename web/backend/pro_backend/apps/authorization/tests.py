"""Tests of Authorization app."""

from django.contrib.auth import get_user_model
from django.test import TestCase
from django.urls import reverse


User = get_user_model()

# Status codes
HTTP_OK = 200
HTTP_CREATED = 201
HTTP_BAD_REQUEST = 400


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

        self.assertTrue(User.objects.filter(username=username).exists())

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

    fixtures = ["pro_backend/fixtures/users.json"]

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
