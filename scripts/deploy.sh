#!/bin/bash
# =============================================================================
# Open-NVR Deployment Script
# Usage: ./scripts/deploy.sh [server_ip] [ssh_port] [ssh_user]
# =============================================================================
set -e

SERVER="${1:-10.0.0.10}"
SSH_PORT="${2:-1313}"
SSH_USER="${3:-open-nvr}"
APP_DIR="/home/open-nvr/apps/open-nvr"
FRONTEND_PORT="${FRONTEND_PORT:-3030}"
BACKEND_PORT="${BACKEND_PORT:-8888}"

echo "=============================="
echo "  Open-NVR Deployment"
echo "=============================="
echo "Server: $SERVER:$SSH_PORT"
echo "User: $SSH_USER"
echo "App Dir: $APP_DIR"
echo ""

# Step 1: Sync code
echo "[1/5] Syncing code..."
rsync -avz --delete \
  --exclude='target' --exclude='node_modules' --exclude='.nuxt' \
  --exclude='.output' --exclude='.env' --exclude='.git' \
  --exclude='docker-compose.server.yml' --exclude='docker-compose.override.yml' \
  -e "ssh -p $SSH_PORT" \
  ./ "$SSH_USER@$SERVER:$APP_DIR/"
echo "Code synced."

# Step 2: Build on server
echo "[2/5] Building..."
ssh -p "$SSH_PORT" "$SSH_USER@$SERVER" bash << REMOTE
  set -e
  source "\$HOME/.cargo/env" 2>/dev/null || true
  cd $APP_DIR

  echo "Building backend..."
  cd backend && SQLX_OFFLINE=true cargo build --release 2>&1 | tail -1
  cd ..

  echo "Building frontend..."
  cd frontend
  npm install --legacy-peer-deps 2>&1 | tail -1
  NUXT_PUBLIC_API_URL=http://$SERVER:$BACKEND_PORT \
  NUXT_PUBLIC_KEYCLOAK_URL=http://$SERVER:8190 \
  NUXT_PUBLIC_KEYCLOAK_REALM=opennvr \
  NUXT_PUBLIC_KEYCLOAK_CLIENT_ID=opennvr-frontend \
  npx nuxt build 2>&1 | tail -1
  cd ..
  echo "Build complete."
REMOTE

# Step 3: Run migrations
echo "[3/5] Running migrations..."
ssh -p "$SSH_PORT" "$SSH_USER@$SERVER" bash << REMOTE
  cd $APP_DIR
  docker exec opennvr-postgres psql -U opennvr -d opennvr -c "DROP TABLE IF EXISTS _sqlx_migrations;" 2>/dev/null
  echo "Migration table reset."
REMOTE

# Step 4: Restart services
echo "[4/5] Restarting services..."
ssh -p "$SSH_PORT" "$SSH_USER@$SERVER" bash << REMOTE
  cd $APP_DIR
  pkill -u $SSH_USER -f "target/release/open-nvr" 2>/dev/null || true
  pkill -u $SSH_USER -f ".output/server/index.mjs" 2>/dev/null || true
  sleep 3

  set -a; source .env; set +a
  nohup ./backend/target/release/open-nvr > backend.log 2>&1 &
  sleep 5

  cd frontend
  nohup env HOST=0.0.0.0 PORT=$FRONTEND_PORT \
    NUXT_PUBLIC_API_URL=http://$SERVER:$BACKEND_PORT \
    NUXT_PUBLIC_KEYCLOAK_URL=http://$SERVER:8190 \
    NUXT_PUBLIC_KEYCLOAK_REALM=opennvr \
    NUXT_PUBLIC_KEYCLOAK_CLIENT_ID=opennvr-frontend \
    node .output/server/index.mjs > ../frontend.log 2>&1 &
  cd ..
  sleep 3
  echo "Services restarted."
REMOTE

# Step 5: Verify
echo "[5/5] Verifying..."
BACKEND=$(curl -sf "http://$SERVER:$BACKEND_PORT/health" 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','?'))" 2>/dev/null || echo "FAIL")
FRONTEND=$(curl -sf -o /dev/null -w "%{http_code}" "http://$SERVER:$FRONTEND_PORT/" 2>/dev/null || echo "FAIL")

echo ""
echo "=============================="
echo "  Deployment Complete"
echo "=============================="
echo "  Backend:  $BACKEND (http://$SERVER:$BACKEND_PORT)"
echo "  Frontend: HTTP $FRONTEND (http://$SERVER:$FRONTEND_PORT)"
echo "=============================="
