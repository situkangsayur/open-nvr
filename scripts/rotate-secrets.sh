#!/bin/bash
# Replace the example credentials that ship in the repo (and are therefore
# public) with generated ones, on a machine that is already running Open-NVR.
#
#   bash scripts/rotate-secrets.sh            # rotate everything still default
#   bash scripts/rotate-secrets.sh --check    # only report what is still default
#
# Run it from the deployment directory (the one holding .env and
# docker-compose.yml). Nothing secret is printed; the new values are written to
# .env only, and the previous .env is kept as .env.bak-<timestamp>.
#
# Rotates: the Postgres role password (Keycloak shares that role), the MinIO
# root credentials, the Keycloak master admin password and the backend client
# secret. Camera credentials in the database are encrypted with
# CREDENTIAL_ENCRYPTION_KEY, which this script never touches — changing that
# key would make stored camera passwords unreadable.
set -euo pipefail

cd "$(dirname "$0")/.."
[ -f .env ] || { echo "No .env here ($PWD). Run this in the deployment directory."; exit 1; }

DEFAULTS=(opennvr_secret "KEYCLOAK_ADMIN_PASSWORD=admin" opennvr-backend-secret "CHANGE-ME" "change-me-generate")
report() {
  local dirty=0
  for k in "${DEFAULTS[@]}"; do
    if grep -q "$k" .env; then echo "still the repo example: $k"; dirty=1; fi
  done
  [ $dirty -eq 0 ] && echo "no example credentials left in .env"
  return 0
}

if [ "${1:-}" = "--check" ]; then report; exit 0; fi

ts=$(date +%Y%m%d-%H%M%S)
cp -a .env ".env.bak-$ts"; chmod 600 ".env.bak-$ts"
echo "backup written: .env.bak-$ts"

export NEW_PG=$(openssl rand -hex 24)
export NEW_MINIO=$(openssl rand -hex 24)
export NEW_KCADM=$(openssl rand -base64 24 | tr -d '/+=')
export NEW_KCCLIENT=$(openssl rand -hex 24)
export NEW_OS="$(openssl rand -base64 18 | tr -d '/+=')Aa1!"

# 1. Postgres role (Keycloak connects with the same role).
docker exec opennvr-postgres psql -U opennvr -d opennvr -v ON_ERROR_STOP=1 \
  -c "ALTER USER opennvr WITH PASSWORD '$NEW_PG'" >/dev/null
echo "postgres: password rotated"

# 2. Keycloak master admin + backend client secret, using the password that is
#    still in .env to authenticate.
OLD_KCADM=$(grep -E '^KEYCLOAK_ADMIN_PASSWORD=' .env | cut -d= -f2-)
KC=/opt/keycloak/bin/kcadm.sh
if docker exec opennvr-keycloak $KC config credentials --server http://localhost:8080 \
     --realm master --user admin --password "$OLD_KCADM" >/dev/null 2>&1; then
  docker exec opennvr-keycloak $KC set-password -r master --username admin \
    --new-password "$NEW_KCADM" >/dev/null
  echo "keycloak: admin password rotated"
  cid=$(docker exec opennvr-keycloak $KC get clients -r opennvr -q clientId=opennvr-backend \
        --fields id --format csv 2>/dev/null | tr -d '"' | head -1)
  if [ -n "${cid:-}" ]; then
    docker exec opennvr-keycloak $KC update "clients/$cid" -r opennvr -s "secret=$NEW_KCCLIENT" >/dev/null
    echo "keycloak: backend client secret rotated"
  else
    echo "keycloak: backend client not found; its secret was left alone"
    NEW_KCCLIENT=""
  fi
else
  echo "keycloak: cannot authenticate as admin; Keycloak secrets left alone"
  NEW_KCADM=""; NEW_KCCLIENT=""
fi

# 3. Write the new values into .env (and add the keys docker-compose expects).
python3 - <<'PY'
import os, re
path = '.env'
text = open(path).read()
get = os.environ.get

def set_key(t, key, value):
    if re.search(rf'(?m)^{key}=.*$', t):
        return re.sub(rf'(?m)^{key}=.*$', f'{key}={value}', t)
    return t.rstrip('\n') + f'\n{key}={value}\n'

pg = os.environ['NEW_PG']
# DATABASE_URL: swap only the password between ':' and '@'
text = re.sub(r'(?m)^(DATABASE_URL=\w+://[^:]+:)[^@]*(@)',
              lambda m: m.group(1) + pg + m.group(2), text)
text = set_key(text, 'POSTGRES_USER', 'opennvr')
text = set_key(text, 'POSTGRES_PASSWORD', pg)
text = set_key(text, 'POSTGRES_DB', 'opennvr')
text = set_key(text, 'MINIO_SECRET_KEY', os.environ['NEW_MINIO'])
text = set_key(text, 'KEYCLOAK_ADMIN_USER', 'admin')
if get('NEW_KCADM'):    text = set_key(text, 'KEYCLOAK_ADMIN_PASSWORD', get('NEW_KCADM'))
if get('NEW_KCCLIENT'): text = set_key(text, 'KEYCLOAK_CLIENT_SECRET', get('NEW_KCCLIENT'))
text = set_key(text, 'OPENSEARCH_ADMIN_PASSWORD', os.environ['NEW_OS'])
open(path, 'w').write(text)
os.chmod(path, 0o600)
PY
echo ".env updated (mode 600)"

# 4. Recreate the containers so they pick up the new environment, then restart
#    the backend, which holds pooled database connections.
docker compose up -d
echo "waiting for postgres and keycloak..."
sleep 20
systemctl restart opennvr-backend 2>/dev/null \
  || kill "$(systemctl show -p MainPID --value opennvr-backend)" 2>/dev/null \
  || echo "restart the backend yourself"

report
echo
echo "Done. Check: journalctl -u opennvr-backend -n 30, then log in to the web UI."
echo "The new values live in .env only. Back up that file somewhere safe."
