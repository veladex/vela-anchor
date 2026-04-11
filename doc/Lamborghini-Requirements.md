

# Version 1.0 - Nodes

(1) Mint Gold Node and Diamond Node NFTs
(2) Launch node purchase functionality
(3) Launch Google Authenticator functionality
(4) Support Cobo third-party system deposit/withdrawal: USDT on TRX and BSC chains, USDC on SOLANA chain

**Node NFT (Single Tier, Passive Earning Design)**

| Node Type | Diamond Node | Gold Node |
| :---- | :---- | :---- |
| **Price** | **$10,000** | **$1,000** |
| **Total Supply** | **600** | **12,000** |
| **Personal Staking Interest Permanent Boost** | **+20%** | **+10%** |
| Node Dividends: Share of **all daily interest from all network staking users** (these two are separate, totaling 15%) | **7.5%** | **7.5%** |
| Token Airdrop Total Released Over 100 Days | **30,000** | **2,250** |
| Daily Airdrop Release (Unclaimed NFT airdrop rewards go to others) | **300** | **22.5** |

**Node Dividend Revenue Estimation:**

Single Gold Node daily dividend = Total daily network staking revenue \* 7.5% / Total supply of 12,000
Single Diamond Node daily dividend = Total daily network staking revenue \* 7.5% / Total supply of 600

# Important Notes:

1. Centralized invitation relationships and on-chain progressive relationships.

(1) Addresses that have not bound relationships in the contract cannot be used for binding by others.
(2) Addresses that have bound relationships in the contract but have not registered/logged in on the centralized system — binding on-chain is allowed, but since the referrer is not registered centrally, the relationship chain is broken. Centralized business permissions are [Cannot Operate]. For such users, a popup should appear at centralized business entry: "The referrer address of this account is missing, this operation cannot be performed." (Supplemented on March 1st)
(3) Only addresses that have [On-chain Binding Relationship] AND [Registered/Logged in on centralized system] can perform centralized business operations after binding the invitation relationship.

# Version 1.1 Iteration - NFT Airdrop Distribution

NFT token release days formula = (Current Time - Purchase Time) / 24, rounded down

Gold Node cumulative pending token airdrop = 22.5 \* Release Days
Diamond Node cumulative pending token airdrop = 300 \* Release Days

User's pending airdrop tokens = Sum of all NFT token airdrops for that account

(1) The calculation of pending token airdrops is for display only. After the on-chain token launch, airdrops will be distributed to user addresses in a single batch.

(2) After unified on-chain distribution, the claimed amount (manually distributed to addresses) will be displayed under [Cumulative Passive Earnings].

(3) Subsequently, users can see pending claimable rewards increasing daily and can actively claim through the contract. After claiming, the statistics are updated on top of (2).

![][image1]

# Version 1.2 - Invitation Link

![][image2]
Added a new display style for invitation addresses showing the address. When copied, it becomes a link. When a new user clicks this link, the invitation popup has the address pre-filled. (Existing users clicking it go to the homepage)

# Version 1.3

## Feature 1: USDC Gifting Whitelist

Client-facing feature name: [**Pioneer Subsidy**], only visible to whitelisted users.

1. Develop a USDC gifting whitelist feature in the admin dashboard. Addresses can be added or removed, and must be registered users on the Vela platform.
2. Whitelisted users can use the USDC gifting feature. Gifted USDC is automatically released at 1% per day based on the cumulative historical gifting amount. Daily release amount = **Cumulative Historical Gift Total** \* 1%, starting at 0:00 each day. Each release checks the remaining total release quota:
   (1) Remaining release quota ≥ Daily release amount → Release standard 1%
   (2) Remaining release quota < Daily release amount → Release only the remaining amount

3. Both the client and admin dashboard can view the total gifted amount, current releasable balance, and each release record.

4. Release starts by default after gifting. Admin dashboard can stop/start the release.

5. Whitelisted users need admin dashboard approval for withdrawals.

6. Separately track node purchase amounts and VELA token subscription amounts for whitelisted users.

## Feature 2: VELA Presale

**VELA Presale Unit Price**: Set in admin dashboard
**Sales Batches**: Sold in multiple rounds
**Sales Quota**: Each round's quantity set in admin dashboard
**Per-Address Participation Limit**: Total account limit of 40,000 tokens, usable across all rounds until depleted.

Everyone can participate in the presale. After purchase, the VELA token amount is shown in the account balance but cannot be withdrawn. When staking goes live, VELA withdrawal will be enabled. (Meeting notes: Latest plan is direct on-chain airdrop, as adding new tokens to Cobo incurs fees)

Admin Dashboard Features: Set quantity and price for each round, start/stop controls.
Airdrop Coordination Process: (1) Get centralized addresses and amounts on-chain, process uniformly; (2) Centralized system provides the data
User input field max is limited by: (1) Max purchasable with personal balance; (2) Max personal presale quota; (3) Max remaining quota for current presale round. Take the smallest value.

Meeting Notes:
1. VELA presale & private sale will stop once staking goes live.
2. No limit on total sales volume. Each round and overall sales quota can be set flexibly. (If exceeding total supply, modify client-side display values)
3. Google Authenticator required when adding rounds and amounts.

## Other Iteration Optimizations:

(1) Redesign the UI settings hamburger menu

## March 8th Frontend Meeting Notes:

Homepage [My Total Assets] = Centralized U (USDT and USDC) + Centralized VELA value in U (price fetched from Pinpet) + User's all on-chain VELA value in U (sum of staked principal in contract, pending interest tokens, and available token balance at user's address)

# Version 2.0 - Staking

#

**February 28th Meeting Amendments:**
1. Each address can only bind 1 NFT
2. New binding time after node transfer is 15 days (since node staking interest boost is only distributed upon claiming)
3. Node dividend rewards can be claimed once per calendar week (Monday to Sunday). Unclaimed rewards go to a designated marketing address (provided by the boss).
4. 10% of claimed interest is sent to the dead address. The node boost portion of interest is also subject to the 10% dead address deduction.
5. Remove "team recent new members count", directly display total team addresses.
6. Per-address stakeable quota needs to be calculated and validated. Product team to supplement the formula.
7. Record the most recent 50 orders and records
8. Frontend needs to fetch the latest interest rates for different periods from on-chain

# I. Token Economics

* **Token Name:** VELA
* **Total Supply:** 1 billion tokens
* **Locked Interest Pool:** 850 million tokens
* **Node Airdrop:** 45 million tokens
* **Marketing Address:** 5 million tokens
* **Initial Liquidity Pool:** 100 million tokens

(To be confirmed with the boss)

# II. Deposit & Earn Rules

**Flow**: User buys VELA from Pinpet (add an entry point) → Deposit into staking contract (whole numbers) → Earn **token-denominated fixed interest** by lock period (no compounding, principal returned at maturity).

* **Lock Periods** (3 options):
  * 7 days: Interest **0.5%**
  * 30 days: Interest **0.7%**
  * 90 days: Interest **1.0%**
* **Per-Deposit Limits**: Minimum 1,000 VELA, maximum 50,000 VELA, must be whole numbers.
* **Reward Claiming & Fees**: Interest is calculated hourly after deposit. Users can claim interest at any time (no threshold). A 10% fee on claimed interest (including node boost interest) is sent to the dead address.
* **Maturity Return**: Full principal + unclaimed interest can be withdrawn at the end of the period.
* **Staking Order Limit**: Users can have a maximum of 50 active staking orders. When exceeded, the button is greyed out with the message: [Current orders have reached 50].  (Client displays up to 50 order status records: Locked, Staking, Completed.)

**Special Note**: The interest rate at the time of staking is locked for that order. Regardless of network-wide rate changes, the rate recorded at the moment of deposit is used for calculations.

**Production halving every 20 million tokens produced from the interest pool (static + dynamic):**

Phase 1: 0–500 million tokens, continuous progressive reduction of 5%

Phase 2: 500–800 million tokens, continuous progressive reduction of 3%

Phase 3: 800–1,000 million tokens, continuous progressive reduction of 2%

# III. Entry Control

* **Global Daily Deposit Cap** (initial): **3 million XXX tokens**
* **Network-Wide Dynamic Adjustment** (only upward, never downward):
  * Previous day **full (≥100%)** → Next day cap increases by **+10% (compounding)** based on previous day's cap
  * Previous day **<100%** → Next day **remains unchanged**
    **Quota depleted: When remaining quota is insufficient for a user deposit, e.g., less than 1,000, it is considered 100% utilized. Button should be greyed out: "Today's quota has ended"**
* **Per-Address Total Staking Cap** (dynamic balance, prevents whale monopoly):
  * **Base Cap**: **50,000 XXX tokens** (≈$2,160)
  * **Dynamic Adjustment (applies equally if staking rises then falls)**:
    * Network-wide staking exceeds 45 million → Per-address cap **100,000 tokens**
    * Network-wide staking exceeds 90 million → Per-address cap **150,000 tokens**

Meeting Notes: Per-address stakeable quota needs to be calculated and validated. Formula to be supplemented.

# IV. Community Rewards

Direct Referral: 5%

**Community Tier 80%**

| Level | Personal Staked VELA | Community Performance (Staked VELA) — downstream only, excluding self | Tier Differential Share (% of subordinate's performance taken by superior) | Same-Tier Reward (Extra share from same-level performance) |
| ----- | ----- | ----- | ----- | ----- |
| L1 | 1,000 | 200,000 | 10% | None |
| L2 | 1,000 | 600,000 | 20% | None |
| L3 | 50,000 | 3,000,000 | 30% | 2% |
| L4 | 50,000 | 6,000,000 | 40% | 2% |
| L5 | 100,000 | 16,000,000 | 50% | 2% |
| L6 | 100,000 | 40,000,000 | 60% | 2% |
| L7 | 150,000 | 100,000,000 | 70% | 2% |

Same-Tier Reward: 2% triggered by deposit interest (static). Formula = Personal claimed deposit earnings \* 2%

Unclaimed portions belong to the root address:
(1) Tier differential unclaimed up to 70% belongs to the root address (boss's address, verify during testing)
(2) Same-tier rewards for L3–L7 that are unclaimed all belong to the root address.

# V. Binding NFT

1. When staking officially launches:
   (1) If a user already holds an NFT, they are forced to the Bind NFT page to select 1 NFT to bind.
   (2) Users without NFTs enter the Bind NFT page after purchasing any 1 NFT.

2. Continue Purchasing
   The binding page has a [Continue Purchasing Other Nodes] button. Clicking it enters the NFT purchase flow. After purchasing, the user returns to the binding page.

3. After binding an NFT, that NFT's status shows [Bound].

4. Every NFT can be transferred, showing a [Gift] button. If it is a bound NFT, an additional confirmation popup appears. (To be supplemented by product team)
