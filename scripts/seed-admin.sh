#!/bin/bash
# =============================================================================
# Open-NVR Admin Seeding Script
# Creates initial admin user and configures Keycloak for direct login
# =============================================================================
set -e

KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8190}"
REALM="opennvr"
ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-admin123}"
KEYCLOAK_ADMIN="${KEYCLOAK_ADMIN:-admin}"
KEYCLOAK_ADMIN_PASSWORD="${KEYCLOAK_ADMIN_PASSWORD:-admin}"

echo "=== Open-NVR Admin Seeding ==="
echo "Keycloak: $KEYCLOAK_URL"
echo "Realm: $REALM"
echo ""

# Get master admin token
echo "Getting admin token..."
TOKEN=$(curl -sf -X POST "$KEYCLOAK_URL/realms/master/protocol/openid-connect/token" \
  -d "client_id=admin-cli" \
  -d "username=$KEYCLOAK_ADMIN" \
  -d "password=$KEYCLOAK_ADMIN_PASSWORD" \
  -d "grant_type=password" | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")

if [ -z "$TOKEN" ]; then
  echo "ERROR: Failed to get Keycloak admin token"
  exit 1
fi
echo "Admin token obtained"

# Enable Direct Access Grants on frontend client
echo "Enabling Direct Access Grants on opennvr-frontend..."
CLIENT_ID=$(curl -sf -H "Authorization: Bearer $TOKEN" \
  "$KEYCLOAK_URL/admin/realms/$REALM/clients?clientId=opennvr-frontend" | \
  python3 -c "import sys,json; c=json.load(sys.stdin); print(c[0]['id'] if c else '')")

if [ -n "$CLIENT_ID" ]; then
  curl -sf -X PUT "$KEYCLOAK_URL/admin/realms/$REALM/clients/$CLIENT_ID" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"directAccessGrantsEnabled": true, "redirectUris": ["*"], "webOrigins": ["*"]}' || true
  echo "Frontend client updated"
fi

# Check if admin user exists, create if not
echo "Checking admin user..."
EXISTING=$(curl -sf -H "Authorization: Bearer $TOKEN" \
  "$KEYCLOAK_URL/admin/realms/$REALM/users?username=$ADMIN_USERNAME" | \
  python3 -c "import sys,json; u=json.load(sys.stdin); print(u[0]['id'] if u else '')")

if [ -z "$EXISTING" ]; then
  echo "Creating admin user: $ADMIN_USERNAME"
  curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"username\": \"$ADMIN_USERNAME\",
      \"email\": \"admin@opennvr.local\",
      \"enabled\": true,
      \"emailVerified\": true,
      \"credentials\": [{
        \"type\": \"password\",
        \"value\": \"$ADMIN_PASSWORD\",
        \"temporary\": false
      }]
    }"

  # Get the new user ID
  EXISTING=$(curl -sf -H "Authorization: Bearer $TOKEN" \
    "$KEYCLOAK_URL/admin/realms/$REALM/users?username=$ADMIN_USERNAME" | \
    python3 -c "import sys,json; u=json.load(sys.stdin); print(u[0]['id'] if u else '')")
  echo "User created: $EXISTING"
else
  echo "Admin user already exists: $EXISTING"
  # Clear required actions and reset password
  curl -sf -X PUT "$KEYCLOAK_URL/admin/realms/$REALM/users/$EXISTING" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"requiredActions": []}' || true
fi

# Assign admin role
echo "Assigning admin role..."
ADMIN_ROLE_ID=$(curl -sf -H "Authorization: Bearer $TOKEN" \
  "$KEYCLOAK_URL/admin/realms/$REALM/roles/admin" | \
  python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))")

if [ -n "$ADMIN_ROLE_ID" ] && [ -n "$EXISTING" ]; then
  curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users/$EXISTING/role-mappings/realm" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "[{\"id\": \"$ADMIN_ROLE_ID\", \"name\": \"admin\"}]" || true
  echo "Admin role assigned"
fi

# Create operator and viewer example users
for ROLE in operator viewer; do
  USER="${ROLE}user"
  echo ""
  echo "Checking $USER..."
  UID_CHECK=$(curl -sf -H "Authorization: Bearer $TOKEN" \
    "$KEYCLOAK_URL/admin/realms/$REALM/users?username=$USER" | \
    python3 -c "import sys,json; u=json.load(sys.stdin); print(u[0]['id'] if u else '')")

  if [ -z "$UID_CHECK" ]; then
    echo "Creating $USER with role: $ROLE"
    curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d "{
        \"username\": \"$USER\",
        \"email\": \"${USER}@opennvr.local\",
        \"enabled\": true,
        \"emailVerified\": true,
        \"credentials\": [{
          \"type\": \"password\",
          \"value\": \"${USER}123\",
          \"temporary\": false
        }]
      }" || true

    UID_CHECK=$(curl -sf -H "Authorization: Bearer $TOKEN" \
      "$KEYCLOAK_URL/admin/realms/$REALM/users?username=$USER" | \
      python3 -c "import sys,json; u=json.load(sys.stdin); print(u[0]['id'] if u else '')")

    ROLE_ID=$(curl -sf -H "Authorization: Bearer $TOKEN" \
      "$KEYCLOAK_URL/admin/realms/$REALM/roles/$ROLE" | \
      python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))")

    if [ -n "$ROLE_ID" ] && [ -n "$UID_CHECK" ]; then
      curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users/$UID_CHECK/role-mappings/realm" \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d "[{\"id\": \"$ROLE_ID\", \"name\": \"$ROLE\"}]" || true
    fi
    echo "$USER created"
  else
    echo "$USER already exists"
  fi
done

echo ""
echo "=== Seeding Complete ==="
echo ""
echo "NOTE: Run database migrations before starting the backend:"
echo "  psql \$DATABASE_URL -f migrations/001_initial.sql"
echo "  psql \$DATABASE_URL -f migrations/002_user_permissions.sql"
echo ""
echo "Users created:"
echo "  admin       / admin123       (role: admin)"
echo "  operatoruser / operatoruser123 (role: operator)"
echo "  vieweruser   / vieweruser123   (role: viewer)"
echo ""
echo "Login at: $KEYCLOAK_URL -> Realm: $REALM"
