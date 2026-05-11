#!/usr/bin/env bash
#
# One-shot devnet bootstrap for SolGrid:
#   1. Switches the Solana CLI to devnet
#   2. Ensures the deployer wallet has SOL
#   3. Generates the verifier keypair if missing
#   4. Builds + deploys the three Anchor programs
#   5. Captures the deployed program IDs into Anchor.toml + .env
#   6. Creates the tSIMD mint via sol-client and writes it to .env
#   7. Pre-funds the demo client wallet with SOL + tSIMD
#
# Run from the repo root:
#   ./scripts/setup-devnet.sh
#
# Re-running is safe — every step is idempotent.

set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"
DATA_DIR="$ROOT/data"
mkdir -p "$DATA_DIR"

step() { printf "\n\033[1;36m▶ %s\033[0m\n" "$*"; }
ok()   { printf "  \033[32m✓\033[0m %s\n" "$*"; }
info() { printf "  %s\n" "$*"; }

# ── 1. Devnet config ──────────────────────────────────────────────────
step "Configuring Solana CLI for devnet"
solana config set --url https://api.devnet.solana.com >/dev/null
ok "RPC = devnet"

# ── 2. Deployer wallet ────────────────────────────────────────────────
step "Ensuring deployer wallet exists"
if [ ! -f "$HOME/.config/solana/id.json" ]; then
  solana-keygen new --no-bip39-passphrase --silent --outfile "$HOME/.config/solana/id.json"
fi
DEPLOYER="$(solana address)"
ok "deployer = $DEPLOYER"

BAL_LAMPORTS=$(solana balance --lamports | awk '{print $1}')
if [ "$BAL_LAMPORTS" -lt 2000000000 ]; then
  info "Airdropping 2 SOL to deployer..."
  solana airdrop 2 || info "Airdrop may be rate-limited — retry later if balance is still 0."
fi
solana balance

# ── 3. Verifier keypair ───────────────────────────────────────────────
step "Verifier authority keypair"
if [ ! -f "$DATA_DIR/verifier-keypair.json" ]; then
  solana-keygen new --no-bip39-passphrase --silent \
    --outfile "$DATA_DIR/verifier-keypair.json"
fi
VERIFIER="$(solana-keygen pubkey "$DATA_DIR/verifier-keypair.json")"
ok "verifier = $VERIFIER"
# Verifier pays rent for receipt PDAs + escrow release fees.
solana transfer "$VERIFIER" 0.5 --allow-unfunded-recipient --keypair ~/.config/solana/id.json >/dev/null 2>&1 || true

# ── 4. Anchor build + deploy ──────────────────────────────────────────
step "Building Anchor programs"
anchor build

step "Deploying programs to devnet"
anchor deploy --provider.cluster devnet

# ── 5. Sync program IDs into .env ─────────────────────────────────────
step "Syncing program IDs into .env"
PROVIDER_REGISTRY_ID=$(solana address -k target/deploy/provider_registry-keypair.json)
JOB_ESCROW_ID=$(solana address -k target/deploy/job_escrow-keypair.json)
COMPUTE_RECEIPTS_ID=$(solana address -k target/deploy/compute_receipts-keypair.json)
ok "provider_registry = $PROVIDER_REGISTRY_ID"
ok "job_escrow        = $JOB_ESCROW_ID"
ok "compute_receipts  = $COMPUTE_RECEIPTS_ID"

ENV_FILE="$ROOT/.env"
if [ ! -f "$ENV_FILE" ]; then
  cp .env.example "$ENV_FILE"
fi
upsert() { # upsert KEY VALUE FILE
  local key="$1" val="$2" file="$3"
  if grep -q "^$key=" "$file"; then
    sed -i "s|^$key=.*|$key=$val|" "$file"
  else
    echo "$key=$val" >>"$file"
  fi
}
upsert PROVIDER_REGISTRY_PROGRAM_ID "$PROVIDER_REGISTRY_ID" "$ENV_FILE"
upsert JOB_ESCROW_PROGRAM_ID        "$JOB_ESCROW_ID"        "$ENV_FILE"
upsert COMPUTE_RECEIPTS_PROGRAM_ID  "$COMPUTE_RECEIPTS_ID"  "$ENV_FILE"

# ── 6. tSIMD mint + client wallet ─────────────────────────────────────
step "Creating tSIMD mint"
cargo run -q -p sol-client -- keygen >/dev/null
cargo run -q -p sol-client -- airdrop --amount-sol 1 || true
MINT_OUTPUT="$(cargo run -q -p sol-client -- setup-mint --decimals 6)"
echo "$MINT_OUTPUT"
MINT_ADDR=$(echo "$MINT_OUTPUT" | grep -m1 '^mint:' | awk '{print $2}')
upsert GRID_TOKEN_MINT "$MINT_ADDR" "$ENV_FILE"
ok "mint = $MINT_ADDR"

step "Minting 1,000,000 tSIMD to demo client"
# 1,000,000 * 10^6 base units
cargo run -q -p sol-client -- mint-to 1000000000000

step "Done. Run docker compose up next."
echo
echo "Quick demo:"
echo "  docker compose up -d postgres redis"
echo "  cargo run -p sol-scheduler &"
echo "  cargo run -p sol-verifier &"
echo "  cargo run -p sol-relayer &"
echo "  cargo run -p sol-agent &"
echo "  cargo run -p sol-client -- create-job --budget-scu 50000"
