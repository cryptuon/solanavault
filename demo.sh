#!/bin/bash

# SolanaVault Demo Script
# This script runs the complete demo sequence

echo "=== SolanaVault Demo ==="
echo ""

echo "1. Running Compression Demo..."
cargo run -p vault-cli -- compress-demo --blocks 245000000:245001000
echo ""

echo "2. Deploying to Vault Network..."
cargo run -p vault-cli -- deploy-to-vault --compressed-blocks compressed_blocks.vault
echo ""

echo "3. Running Cost Analysis..."
cargo run -p vault-cli -- cost-analysis
echo ""

echo "=== Demo Complete ==="
echo "To test the RPC proxy, run: cargo run -p vault-rpc-proxy"
echo "The proxy will be available at http://127.0.0.1:3030"