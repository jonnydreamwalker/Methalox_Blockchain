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

   
**IMPORTANT LEGAL DISCLAIMER – READ CAREFULLY**

MethaloxChain is experimental software and a technological research project. All materials, including this document, the repository, and associated code, are provided "as is" for informational and educational purposes only.

- **Not Financial Advice or Investment Offer**: MethaloxChain and XSX are not securities, commodities, or investment products. No information herein constitutes financial, investment, legal, tax, or other advice. Participation in the network does not entitle any person to profits, dividends, returns, or any economic benefit.
- **High Risk and No Guarantees**: Blockchain technologies involve substantial risk, including complete loss of value or functionality. There are no representations or warranties (express or implied) regarding performance, security, availability, or future development. The project may be modified, discontinued, or rendered inoperable at any time without notice.
- **Regulatory Compliance**: MethaloxChain is not registered with any financial or securities regulatory authority worldwide. Users are solely responsible for determining and complying with all applicable laws, regulations, and restrictions in their jurisdiction. Use may be prohibited in certain jurisdictions.
- **Liability Limitation**: To the maximum extent permitted by law, the developers, contributors, founders, and associated parties disclaim all liability for any direct, indirect, incidental, consequential, or punitive damages arising from use of MethaloxChain, XSX, or related materials, including but not limited to loss of funds, data, or opportunity.
- **No Reliance**: Users must conduct independent due diligence and not rely on any statement, omission, or implication in project materials.
- **Governing Law**: This disclaimer shall be governed by the laws of [Your Jurisdiction, e.g., Wyoming, USA], without regard to conflict of law principles.

By accessing, downloading, using, or participating in MethaloxChain, you acknowledge that you have read, understood, and agree to this disclaimer in its entirety.

Last updated: December 29, 2025
