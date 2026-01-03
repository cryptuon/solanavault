#!/bin/bash
#
# SolanaVault Smart Contract Deployment Script
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROGRAMS_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Default to devnet
CLUSTER="${1:-devnet}"

log_info "Deploying SolanaVault programs to $CLUSTER"

# Check prerequisites
if ! command -v anchor &> /dev/null; then
    log_error "Anchor CLI not found. Install with: cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked"
    exit 1
fi

if ! command -v solana &> /dev/null; then
    log_error "Solana CLI not found. Install from: https://docs.solana.com/cli/install-solana-cli-tools"
    exit 1
fi

# Set cluster
log_info "Setting cluster to $CLUSTER"
solana config set --url "$CLUSTER"

# Check wallet balance
BALANCE=$(solana balance | awk '{print $1}')
log_info "Wallet balance: $BALANCE SOL"

if (( $(echo "$BALANCE < 5" | bc -l) )); then
    log_warn "Low balance. You may need more SOL for deployment."
    if [ "$CLUSTER" = "devnet" ]; then
        log_info "Requesting airdrop..."
        solana airdrop 2
        sleep 5
    fi
fi

# Build programs
log_info "Building programs..."
cd "$PROGRAMS_DIR"
anchor build

# Deploy programs in order
PROGRAMS=("vault-token" "vault-staking" "vault-rewards" "vault-governance")

for PROGRAM in "${PROGRAMS[@]}"; do
    log_info "Deploying $PROGRAM..."

    PROGRAM_SO="target/deploy/${PROGRAM//-/_}.so"
    KEYPAIR="target/deploy/${PROGRAM//-/_}-keypair.json"

    if [ ! -f "$PROGRAM_SO" ]; then
        log_error "Program binary not found: $PROGRAM_SO"
        exit 1
    fi

    # Deploy
    PROGRAM_ID=$(solana program deploy "$PROGRAM_SO" --program-id "$KEYPAIR" --output json | jq -r '.programId')

    log_info "$PROGRAM deployed at: $PROGRAM_ID"
done

log_info "Deployment complete!"

# Output program IDs
echo ""
echo "============================================"
echo "Program IDs for $CLUSTER:"
echo "============================================"
for PROGRAM in "${PROGRAMS[@]}"; do
    KEYPAIR="target/deploy/${PROGRAM//-/_}-keypair.json"
    PROGRAM_ID=$(solana-keygen pubkey "$KEYPAIR")
    echo "$PROGRAM: $PROGRAM_ID"
done
echo "============================================"

# Save to file
OUTPUT_FILE="deployed-$CLUSTER.json"
echo "{" > "$OUTPUT_FILE"
first=true
for PROGRAM in "${PROGRAMS[@]}"; do
    KEYPAIR="target/deploy/${PROGRAM//-/_}-keypair.json"
    PROGRAM_ID=$(solana-keygen pubkey "$KEYPAIR")
    if [ "$first" = true ]; then
        first=false
    else
        echo "," >> "$OUTPUT_FILE"
    fi
    echo "  \"${PROGRAM//-/_}\": \"$PROGRAM_ID\"" >> "$OUTPUT_FILE"
done
echo "}" >> "$OUTPUT_FILE"

log_info "Program IDs saved to $OUTPUT_FILE"
