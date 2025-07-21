
import requests
import argparse
import sys

def login(base_url, username, password):
    url = f"{base_url}/api/v1/auth/login"
    try:
        response = requests.post(
            url,
            json={"username": username, "password": password},
            headers={"Content-Type": "application/json"}
        )
        response.raise_for_status()
        return response.json().get("payload").get("access_token")
    except requests.exceptions.RequestException as e:
        print(f"login failed: {e}")
        return None


def delete_expired_tokens(base_url, access_token):
    url = f"{base_url}/api/v1/maintenance/delete-expired-jwt"
    try:
        response = requests.get(
            url,
            headers={"Authorization": f"Bearer {access_token}"}
        )
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        print(f"failed to delete expired jwt: {e}")
        return None


def main():
    parser = argparse.ArgumentParser(description="Delete expired JWT refresh tokens from the database.")
    parser.add_argument("base_url", help="Base URL of the API (e.g., http://localhost:8080)")
    parser.add_argument("username", help="Admin username")
    parser.add_argument("password", help="Admin password")
    args = parser.parse_args()

    base_url = args.base_url.rstrip("/")

    print(f"Logging in as {args.username}")
    access_token = login(base_url, args.username, args.password)
    if not access_token:
        print("Failed to obtain access token.")
        sys.exit(1)

    print("Authenticated. Deleting expired tokens...")
    result = delete_expired_tokens(base_url, access_token)
    print(f"response={result}") if result else sys.exit(1)


if __name__ == "__main__": main()
