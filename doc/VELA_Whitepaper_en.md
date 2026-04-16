# VELA Contract Fairness and Security Whitepaper

> This document is intended for all VELA community users. It provides a comprehensive explanation of the design and guarantees behind the VELA smart contract in terms of security, fairness, and transparency.

---

## 1. Third-Party Authoritative Audit

The VELA smart contract has undergone a rigorous audit by **CertiK**, the world's top-ranked blockchain security auditing firm. CertiK has provided security audit services for over 4,000 blockchain projects, with clients including Aave, Polygon, and BNB Chain. It is the most authoritative security auditing institution in the industry.

The audit covers:
- Comprehensive review of contract code logic
- Common vulnerability scanning (reentrancy attacks, overflows, permission bypasses, etc.)
- Backdoor detection and fund security verification
- Confirmation of the security and reliability of fund transfer logic

Audit conclusion: The contract code contains no security vulnerabilities, no backdoors, and no hidden admin withdrawal functions.

The audit report is publicly available: **https://skynet.certik.com/projects/vela**

Anyone can view VELA's security score and audit details on the CertiK official website.

---

## 2. Fully Open-Source Code

VELA firmly believes in "code is law." We make all code fully public and open to scrutiny by the entire community:

**Smart Contract Source Code**
- The contract code is hosted on GitHub and can be viewed and reviewed by anyone
- Repository: https://github.com/veladex/vela-anchor

**Frontend Website Code**
- To eliminate any concerns about website security, we have also open-sourced the compiled frontend code
- Repository: https://github.com/veladex/vela-website

This means VELA has no "black boxes" — from the on-chain contract to the web interface you use, everything is transparent.

---

## 3. On-Chain Code Verification — Seeing Is Believing

Is the open-source code truly identical to the program running on-chain? You don't need to take our word for it — you can verify it yourself.

Solana provides an official code verification tool, `solana-verify`, which anyone can use to run the following command to perform a byte-by-byte comparison between the source code on GitHub and the program deployed on-chain:

```bash
solana-verify verify-from-repo \
    --url https://api.mainnet-beta.solana.com \
    --program-id FW6P7G9yPBqGAGsZ6Aa7upC9whF69QMH4ZJaBJjFsLVK \
    https://github.com/veladex/vela-anchor
```

Verification result:

```
Executable Program Hash from repo: 5a9221595cb0f03287e8f58c1613173316ff036f274668f4db716d8eab2fc343
On-chain Program Hash: 5a9221595cb0f03287e8f58c1613173316ff036f274668f4db716d8eab2fc343
Program hash matches ✅
```

The two hash values are identical, proving that the program running on-chain is exactly the same as the compiled result of the open-source code, with no tampering of any kind.

---

## 4. Verifiable Contract Code

Once deployed, the contract code is fixed on the Solana blockchain and generates a unique program hash. This hash has been certified by the CertiK audit and is publicly recorded. This means:

- Any modification to the contract code will produce a completely different program hash, which cannot pass CertiK's audit verification
- Even redeploying with the same source code will generate a new contract program address, which likewise will not match the audited record
- Anyone in the community can use the `solana-verify` tool at any time to compare the on-chain program with the open-source code and confirm they are consistent

Every stake you make and every reward you claim is executed strictly according to the rules written in the on-chain code, with continuous assurance from the CertiK audit.

---

## 5. Fund Security — No One Can Touch Your Money

This is the question users care about most, and we give the clearest possible answer:

**There is no function in the contract that allows an admin to transfer user funds.**

- Your funds are stored in dedicated accounts (PDAs) on the blockchain, controlled by the contract program
- All fund operations (staking, redemption, claiming rewards) must be signed by your own wallet
- Admin permissions are limited to initial configuration (such as creating NFT collections and initializing the referral system) and are completely unrelated to fund transfers
- The 10% tax deducted when claiming interest is sent directly to Solana's burn address (a black hole address) and does not enter anyone's wallet

Even if the VELA website shuts down, your funds remain safely on the blockchain and can be retrieved at any time through on-chain interaction. For detailed instructions on how to retrieve your funds, please refer to the Fund Security Guide.

---

## 6. Fair and Transparent Rules

### Staking Rules Are Public and Transparent

| Staking Period | Daily Rate | Staking Range |
|----------------|------------|---------------|
| 7 days | 0.5% | 1,000 - 50,000 VELA |
| 30 days | 0.7% | 1,000 - 50,000 VELA |
| 90 days | 1.0% | 1,000 - 50,000 VELA |

The above rates are hardcoded in the contract and are completely identical for all users. There are no "special treatments" or "hidden rules."

### Tax Destination Is Transparent

The 10% tax deducted when claiming interest is 100% sent to the burn address on the Solana blockchain. This process is executed entirely on-chain and can be verified in real time by anyone using a block explorer (such as Solscan or Solana Explorer).

### NFT Supply Is Limited and Public

- Diamond NFT maximum supply: 600
- Gold NFT maximum supply: 12,000

The supply caps are hardcoded in the contract and cannot be increased.

---

## 7. No Website Dependency — True Decentralization

VELA is a fully decentralized protocol deployed on the Solana blockchain. The website is simply a convenient interface for you to interact with; the contract itself runs independently on the blockchain.

We provide open-source interaction scripts so that even without the website, you can interact directly with the contract to perform all operations. The script is located at `scripts/stake-example.js` in the project repository and can be used as follows:

1. Install Node.js (v16 or above)
2. Clone the repository and install dependencies
   ```bash
   git clone https://github.com/veladex/vela-anchor.git
   cd vela-anchor/scripts
   npm install
   ```
3. Configure your information in `stake-example.js`:
   - Replace `privateKeyString` with your wallet private key (bs58 format)
   - Change the RPC address in `connection` to the Solana mainnet RPC (e.g., `https://api.mainnet-beta.solana.com`)
   - Change `mintAddress` to the mainnet VELA token address
4. Call the corresponding function as needed:
   ```bash
   node stake-example.js
   ```

The script includes complete operation examples:

| Function | Action | Description |
|----------|--------|-------------|
| `addReferral` | Bind referrer | Must bind a referrer before staking |
| `createStake` | Create stake | Specify amount and period type (1=7 days, 2=30 days, 3=90 days) |
| `getMyStakingOrders` | Query staking orders | View all active stakes |
| `claimInterest` | Claim interest | Claim accumulated interest for a specified order |
| `unstake` | Unstake | Redeem principal and remaining interest after maturity |

All operations only require your own wallet signature — no third-party authorization is needed.

> Note: If you are not familiar with technical operations, you can ask any developer in the community to help you verify and execute — because the code is fully open-source, anyone can confirm the safety of the scripts.

---

## 8. On-Chain Data Is Publicly Accessible

All VELA transaction records, fund transfers, and contract states are recorded on the Solana blockchain and can be viewed by anyone using the following tools:

- **Solana Explorer**: https://explorer.solana.com
- **Solscan**: https://solscan.io

You can view at any time:
- All historical transactions of the contract
- Real-time balance of the liquidity pool
- Every tax burn record
- Your personal staking status and earnings

---

## 9. Frequently Asked Questions

**Q: Could the team run away with the funds?**
A: No. There is no admin withdrawal function in the contract. Funds can only be operated by the user themselves through wallet signatures. Even if the team disbands, your funds remain safely on-chain and can be retrieved at any time.

**Q: Could the contract be secretly upgraded to add a backdoor?**
A: No. The contract code has been audited by CertiK and a unique program hash has been generated. Any code modification will change the hash and cannot pass audit verification; even redeploying from source code will produce a new program address, which will likewise not be recognized by CertiK. You can independently verify the consistency between the on-chain code and the open-source repository at any time using the `solana-verify` tool.

**Q: Could the interest rates be secretly modified?**
A: No. The rate parameters are hardcoded in the contract code — they are not modifiable variables. Changing the rates would require deploying an entirely new contract (with a new address), and existing stakes in the old contract would not be affected in any way.

**Q: Is my referral reward calculated fairly?**
A: The referral reward calculation logic is written entirely in the on-chain contract and applies equally to everyone. You can review the open-source code or the audit report to verify the calculation rules.

**Q: Could NFTs be infinitely minted to dilute value?**
A: No. The Diamond NFT cap is 600 and the Gold NFT cap is 12,000. These numbers are hardcoded in the contract and cannot be modified.

**Q: If I'm not technically savvy, how can I confirm these claims are true?**
A: You can ask any friend familiar with Solana development to help you verify, or refer to the CertiK audit report. Our code is fully open-source and can withstand scrutiny from anyone.

**Q: Could the contract be hacked?**
A: The VELA contract has been professionally audited by CertiK, the world's top-ranked firm. The code is developed using Anchor, the most mature framework in the Solana ecosystem, and follows the highest industry security standards. The code is also fully open-source and continuously subject to review by security researchers worldwide.

**Q: Could the website steal my private key?**
A: No. The VELA contract and website do not store or access your private key. All transaction signing is done locally in your wallet (such as Phantom). The frontend code is also open-source and can be inspected by anyone. Please make sure to keep your private key or seed phrase safe — it is the sole guarantee of your fund security.

---

## Summary

| Security Dimension | VELA's Approach |
|--------------------|-----------------|
| Code Security | Audited by world's #1 CertiK — zero vulnerabilities, zero backdoors |
| Code Transparency | Contract + frontend fully open-source |
| Code Authenticity | On-chain verification via solana-verify — hashes match |
| Contract Verifiability | CertiK audit locks the hash — any change is immediately detectable |
| Fund Autonomy | Controlled by user wallet signatures — no admin withdrawals |
| Rule Fairness | Interest rates, tax rates, and NFT caps all hardcoded |
| Tax Transparency | 10% tax burned directly on-chain — publicly verifiable |
| Decentralization | No website dependency — open-source scripts for direct interaction |

*VELA's security is not guaranteed by promises, but by code, audits, and the transparency of the blockchain. We welcome scrutiny and verification from every user and security researcher.*
