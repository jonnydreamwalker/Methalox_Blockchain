
# MethaloxChain ![Methalox Logo ](https://github.com/user-attachments/assets/f615e50d-d061-45bc-b555-38bc87c3be9b)


MethaloxChain is a high-performance Layer 1 blockchain protocol featuring VRF-based Proof-of-Stake consensus, configurable block intervals targeting approximately one second, a fixed supply cap of 105 billion XSX coins with controlled dynamic tail emission, differential transaction fee structures, and native multi-asset support.

The core node implementation is designed for efficiency, security, and long-term economic sustainability. XSX is the native cryptocurrency of the network, serving as the primary unit for transaction fees, staking, and economic mechanisms.

Wallet and advanced user interfaces are under active development.

## Key Features

- **Consensus**: VRF-based leader selection for fast and fair block production.
- **Block Time**: ~1 second (configurable).
- **Supply Model**: 105 billion XSX coin cap with dynamic tail emission:
  - Base reward: 50 XSX per block.
  - Additional minting scaled to shortfall from cap (one XSX minted for every 10,000,000 below cap).
  - Rewards distributed pro-rata to all stakers based on stake proportion.
- **Transaction Fees**: 0.1% fee with 50/50 split:
  - 50% to block-producing validator (full retention).
  - 50% founder rake with 1% burn on XSX portion for targeted deflation.
- **Multi-Asset Native Support**: Fees and balances handled per asset.
- **P2P Networking**: libp2p with gossipsub for efficient block propagation.

## Quick Start (Node Operators)

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Clone and run the node:
```bash
git clone https://github.com/jonnydreamwalker/Methalox_Blockchain.git
cd Methalox_Blockchain
cargo build --release
nohup ./target/release/methalox_end_game > methalox.log 2>&1 &
```

Monitor logs:
```bash
tail -f methalox.log
```

Open ports 9933 (RPC) and 40000–60000 (P2P) in your firewall/security list for public access.

## IMPORTANT LEGAL DISCLAIMER – READ CAREFULLY

MethaloxChain is experimental software and a technological research project. All materials, including this document, the repository, and associated code, are provided "as is" for informational and educational purposes only.

- **Not Financial Advice or Investment Offer**: MethaloxChain and XSX are not securities, commodities, or investment products. No information herein constitutes financial, investment, legal, tax, or other advice. Participation in the network does not entitle any person to profits, dividends, returns, or any economic benefit.

- **High Risk and No Guarantees**: Blockchain technologies involve substantial risk, including complete loss of value or functionality. There are no representations or warranties (express or implied) regarding performance, security, availability, or future development. The project may be modified, discontinued, or rendered inoperable at any time without notice.

- **Regulatory Compliance**: MethaloxChain is not registered with any financial or securities regulatory authority worldwide. Users are solely responsible for determining and complying with all applicable laws, regulations, and restrictions in their jurisdiction. Use may be prohibited in certain jurisdictions.

- **Liability Limitation**: To the maximum extent permitted by law, the developers, contributors, founders, and associated parties disclaim all liability for any direct, indirect, incidental, consequential, or punitive damages arising from use of MethaloxChain, XSX, or related materials, including but not limited to loss of funds, data, or opportunity.

- **No Reliance**: Users must conduct independent due diligence and not rely on any statement, omission, or implication in project materials.

- **Governing Law**: This disclaimer shall be governed by the laws of Wyoming, United States of America, without regard to conflict of law principles.

By accessing, downloading, using, or participating in MethaloxChain, you acknowledge that you have read, understood, and agree to this disclaimer in its entirety.

**Last updated**: January 1, 2026

---

