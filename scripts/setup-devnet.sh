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
  info "Deployer needs ≥ 2 SOL. Trying programmatic faucets (per faucet.solana.com)..."

  # Method 1: Solana CLI airdrop. Rate-limited, but cheap to try first.
  info "[1/2] solana airdrop 2 $DEPLOYER --url devnet"
  solana airdrop 2 "$DEPLOYER" --url devnet || true
  sleep 2
  BAL_LAMPORTS=$(solana balance --lamports | awk '{print $1}')

  # Method 2: Proof-of-Work faucet. No rate limits; mines until target is hit.
  if [ "$BAL_LAMPORTS" -lt 2000000000 ]; then
    info "[2/2] devnet-pow mine (PoW faucet — no rate limits, takes a few minutes)"
    if ! command -v devnet-pow >/dev/null 2>&1; then
      info "Installing devnet-pow (one-time)..."
      cargo install devnet-pow
    fi
    # -t is the target balance in lamports; 5e9 = 5 SOL, comfortably above the
    # 2 SOL deploy minimum so reruns don't have to mine again.
    devnet-pow mine -d 3 --reward 0.02 --no-infer -t 5000000000
  fi

  # Method 3 (manual): run a local validator with unlimited airdrops:
  #   solana-test-validator
  #   solana airdrop 100 $DEPLOYER --url localhost
  # Not wired up here because it requires switching the cluster off devnet.
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
# Funding is deferred until after the program deploys (which are the
# expensive step) so the deployer keeps maximum SOL available for rent.

# ── 4. Anchor build + deploy ──────────────────────────────────────────
# Align declare_id!() in each lib.rs and the [programs.*] tables in
# Anchor.toml with the actual keypairs under target/deploy/. Without this
# step the deployed program asserts DeclaredProgramIdMismatch (0x1004).
step "Syncing program IDs (anchor keys sync)"
anchor keys sync

step "Building Anchor programs"
anchor build

# --no-idl skips uploading the IDL account after each deploy. The IDL
# upload is what was failing with 0x1004, and even on success it costs
# ~0.1 SOL of rent per program plus several tx fees — none of which the
# Rust clients in this repo need at runtime (they use sol-onchain::ids).
step "Deploying programs to devnet (--no-idl)"
anchor deploy --provider.cluster devnet --no-idl

# Fund the verifier *after* deploys so the deployer has full balance for
# rent. 0.05 SOL covers ~25 receipt-PDA rents (~0.002 each) plus fees,
# which is plenty for a demo run. Idempotent: only tops up if low.
VERIFIER_BAL=$(solana balance "$VERIFIER" --lamports 2>/dev/null | awk '{print $1}')
VERIFIER_BAL=${VERIFIER_BAL:-0}
if [ "$VERIFIER_BAL" -lt 50000000 ]; then
  info "Funding verifier with 0.05 SOL..."
  solana transfer "$VERIFIER" 0.05 --allow-unfunded-recipient \
    --keypair ~/.config/solana/id.json >/dev/null
fi

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
# Fund the demo client from the deployer (devnet airdrop RPC is rate-limited).
# 0.05 SOL covers mint+ATA creation (~0.0035), escrow PDA rent (~0.002),
# and a few dozen tx fees — far more than a demo needs.
CLIENT_PUBKEY="$(solana-keygen pubkey "$DATA_DIR/client-keypair.json")"
CLIENT_BAL=$(solana balance "$CLIENT_PUBKEY" --lamports 2>/dev/null | awk '{print $1}')
CLIENT_BAL=${CLIENT_BAL:-0}
if [ "$CLIENT_BAL" -lt 50000000 ]; then
  info "Funding demo client ($CLIENT_PUBKEY) with 0.05 SOL..."
  solana transfer "$CLIENT_PUBKEY" 0.05 --allow-unfunded-recipient \
    --keypair ~/.config/solana/id.json >/dev/null
fi
MINT_OUTPUT="$(cargo run -q -p sol-client -- setup-mint --decimals 6)"
echo "$MINT_OUTPUT"
MINT_ADDR=$(echo "$MINT_OUTPUT" | grep -m1 '^mint:' | awk '{print $2}')
upsert GRID_TOKEN_MINT "$MINT_ADDR" "$ENV_FILE"
ok "mint = $MINT_ADDR"

step "Minting 1,000,000 tSIMD to demo client"
# 1,000,000 * 10^6 base units
cargo run -q -p sol-client -- mint-to 1000000000000

# Reclaim rent from any leftover deploy buffers (a `solana program deploy`
# that fails mid-write leaves a funded buffer account behind that can hold
# 1+ SOL of locked rent).
step "Reclaiming SOL from orphaned deploy buffers"
solana program close --buffers --bypass-warning 2>/dev/null || \
  info "(no orphaned buffers, or close not supported by this CLI version)"

step "Done. Run docker compose up next."
echo
echo "Quick demo:"
echo "  docker compose up -d postgres redis"
echo "  cargo run -p sol-scheduler &"
echo "  cargo run -p sol-verifier &"
echo "  cargo run -p sol-relayer &"
echo "  cargo run -p sol-agent &"
echo "  cargo run -p sol-client -- create-job --budget-scu 50000"
