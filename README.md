# MethaloxChain

**MethaloxChain** is a high-performance Layer 1 blockchain protocol featuring VRF-based Proof-of-Stake consensus, 1-second block intervals, a 105 billion XSX fixed floating supply cap with dynamic tail emission, differential transaction fee burn, and native multi-asset support.

The core node implementation is designed for efficiency, security, and long-term economic sustainability.

**Wallet and advanced user interfaces are under active development.**

## Key Features

- **Consensus**: VRF-based leader selection for fast and fair block production.
- **Block Time**: ~1 second (configurable).
- **Supply Model**: 105 billion XSX cap with dynamic tail emission and reactivation threshold (±5% band) for controlled scarcity.
- **Transaction Fees**: 0.1% fee with 50/50 split:
  - 50% to block-producing validator (full retention).
  - 50% founder rake with 99.9% burn on XSX portion for targeted deflation.
- **Multi-Asset Native Support**: Fees and balances handled per asset.
- **P2P Networking**: libp2p with gossipsub for efficient block propagation.

## Quick Start (Node Operators)

1. Install Rust:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
