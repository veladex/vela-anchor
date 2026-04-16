const bs58 = require('bs58');
const { Keypair, Connection, PublicKey, SystemProgram } = require('@solana/web3.js');
const { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } = require('@solana/spl-token');
const anchor = require('@coral-xyz/anchor');

//const idl = require("./idl/vela_anchor.json"); // testnet
const idl = require("./idl/vela_anchor_main.json");  // mainnet

const VELA_PROGRAM_ID = new PublicKey(idl.address);
const WALLET_ID_MAPPING_SEED = "wallet_id_mapping";
const REFERRAL_MANAGER_SEED = "referral_manager";
const USER_STAKE_SEED = "user_stake";
const GLOBAL_STATE_SEED = "global_state";
const LOCKED_TOKEN_VAULT_SEED = "locked_token_vault_seed";

// Some helper functions, not intended for direct user use
function getReferralStoragePdas() {
    const storageAccounts = [];
    for (let i = 1; i <= 9; i++) {
        const [storagePda] = PublicKey.findProgramAddressSync(
            [Buffer.from('referral_storage'), Buffer.from([i])],
            VELA_PROGRAM_ID,
        );
        storageAccounts.push(storagePda);
    }
    return storageAccounts;
}

function getWalletMappingPda(walletPublicKey) {
    const [pda] = PublicKey.findProgramAddressSync(
        [WALLET_ID_MAPPING_SEED, walletPublicKey.toBuffer()], 
        VELA_PROGRAM_ID,
    );
    return pda;
}

function getVelaProgram(connection, wallet) {
    const provider = new anchor.AnchorProvider(
        connection, new anchor.Wallet(wallet), { commitment: connection.commitment || 'confirmed'}
    ); 
    return new anchor.Program(idl, provider);
}

async function getWalletId(program, walletPublicKey) {
    const pda = getWalletMappingPda(walletPublicKey);
    return (await program.account.walletIdMapping.fetch(pda)).referralId;
}


// Bind referrer
async function addReferral(connection, wallet, referralAddress) {
    const program = getVelaProgram(connection, wallet); 
    const walletMappingPda = getWalletMappingPda(wallet.publicKey);
    const storages = getReferralStoragePdas(connection);  
    const [manager] = PublicKey.findProgramAddressSync([REFERRAL_MANAGER_SEED], VELA_PROGRAM_ID);
    const [globalStatePda] = PublicKey.findProgramAddressSync([GLOBAL_STATE_SEED], VELA_PROGRAM_ID);
    const referralFeeWallet = (await program.account.globalState.fetch(globalStatePda)).referralFeeWallet;
    const referralId = await getWalletId(program, new PublicKey(referralAddress));
    
    try {
        const tx = await program.methods
            .addReferral(
                wallet.publicKey,
                referralId
            )
            .accounts({
                payer: wallet.publicKey,
                walletSigner: wallet.publicKey,
                manager,
                storage1: storages[0],
                storage2: storages[1],
                storage3: storages[2],
                storage4: storages[3],
                storage5: storages[4],
                storage6: storages[5],
                storage7: storages[6],
                storage8: storages[7],
                storage9: storages[8],
                walletMapping: walletMappingPda,
                globalState: globalStatePda,
                referralFeeWallet,
                systemProgram: SystemProgram.programId,
            })
            .rpc(); 
        console.log(`Bind referrer success! tx: ${tx}`);
    } catch (err) {
        console.error("Bind referrer failed: referrer already exists.");
    }
}

// Stake. Specify the token amount and period type
async function createStake(connection, wallet, tokenMint, amount, periodType) {
    const program = getVelaProgram(connection, wallet); 
    const [userStakeAccount] = PublicKey.findProgramAddressSync([USER_STAKE_SEED, wallet.publicKey.toBuffer()], VELA_PROGRAM_ID);
    const [globalState] = PublicKey.findProgramAddressSync([GLOBAL_STATE_SEED], VELA_PROGRAM_ID);
    const walletMapping = getWalletMappingPda(wallet.publicKey);
    const storages = getReferralStoragePdas(connection);
    const [lockedVault] = PublicKey.findProgramAddressSync([LOCKED_TOKEN_VAULT_SEED, tokenMint.toBuffer()], VELA_PROGRAM_ID);
    const userTokenAccount = getAssociatedTokenAddressSync(tokenMint, wallet.publicKey);
    const vaultTokenAccount = (await program.account.lockedTokenVault.fetch(lockedVault)).vaultTokenAccount;
      
    try {
        const tx = await program.methods
            .createStake(new anchor.BN(amount), periodType)
            .accounts({
                user: wallet.publicKey,
                userStakeAccount,
                globalState,
                userTokenAccount,
                lockedVault,
                vaultTokenAccount,
                walletMapping,
                storage1: storages[0],
                storage2: storages[1],
                storage3: storages[2],
                storage4: storages[3],
                storage5: storages[4],
                storage6: storages[5],
                storage7: storages[6],
                storage8: storages[7],
                storage9: storages[8],
                userState: null,
                nftBindingState: null,
                userNftAccount: null,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId
            })
            .rpc();
        console.log(`Stake success! tx: ${tx}`);
    } catch (err) {
        console.error("Stake failed. Possible reasons: 1) Wallet balance below 0.03 SOL. 2) Insufficient token balance for staking. 3) Must bind a referrer before staking. 4) Daily staking quota reached, try again tomorrow.");
    }
}

// Returns the list of active staking orders. The contract reserves 20 order slots per user, allowing up to 20 concurrent stakes. Order status: 0 = not staked, 1 = staking, 2 = unstaked
async function getMyStakingOrders(connection, wallet) {
    const program = getVelaProgram(connection, wallet);
    const [userStakePda] = PublicKey.findProgramAddressSync([USER_STAKE_SEED, wallet.publicKey.toBuffer()], VELA_PROGRAM_ID);
    try {
        const stakeAccount = await program.account.userStakeAccount.fetch(userStakePda);
        const orders = stakeAccount.orders
            .map((order, i) => ({ 
                ...order, 
                index: i, 
                endTime: new Date(order.endTime.toNumber() * 1000)
            }))
            .filter(order => order.status === 1);
        return orders;
    } catch (err) {
        console.error("Stake account not yet created");
    }
} 

// Claim staking interest for an order. Requires the order index (order.index)
async function claimInterest(connection, wallet, tokenMint, orderIndex) {
    const program = getVelaProgram(connection, wallet);
    const [userStakePda] = PublicKey.findProgramAddressSync([USER_STAKE_SEED, wallet.publicKey.toBuffer()], VELA_PROGRAM_ID);
    const [globalState] = PublicKey.findProgramAddressSync([GLOBAL_STATE_SEED], VELA_PROGRAM_ID);
    const walletMapping = getWalletMappingPda(wallet.publicKey);
    const storages = getReferralStoragePdas(connection);
    const userTokenAccount = getAssociatedTokenAddressSync(tokenMint, wallet.publicKey);
    const deadAddressTokenAccount = getAssociatedTokenAddressSync(tokenMint, new PublicKey('1nc1nerator11111111111111111111111111111111'), true);
    const [lockedVault] = PublicKey.findProgramAddressSync([LOCKED_TOKEN_VAULT_SEED, tokenMint.toBuffer()], VELA_PROGRAM_ID);
    const vaultTokenAccount = (await program.account.lockedTokenVault.fetch(lockedVault)).vaultTokenAccount;

    try {
        const tx = await program.methods
            .claimInterest(orderIndex)
            .accounts({
                user: wallet.publicKey,
                userStakeAccount: userStakePda,
                globalState,
                userTokenAccount,
                lockedVault,
                vaultTokenAccount,
                deadAddressTokenAccount,
                walletMapping,
                storage1: storages[0],
                storage2: storages[1],
                storage3: storages[2],
                storage4: storages[3],
                storage5: storages[4],
                storage6: storages[5],
                storage7: storages[6],
                storage8: storages[7],
                storage9: storages[8],
                userState: null,
                nftBindingState: null,
                userNftAccount: null,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId
            })
            .rpc();
        console.log(`Claim interest success! tx: ${tx}`);
    } catch (err) {
        console.error("Claim interest failed, no interest available to claim yet");
    }
}

// Unstake an expired order. Requires the order index (order.index)
async function unstake(connection, wallet, tokenMint, orderIndex) {
    const program = getVelaProgram(connection, wallet);
    const [userStakePda] = PublicKey.findProgramAddressSync([USER_STAKE_SEED, wallet.publicKey.toBuffer()], VELA_PROGRAM_ID);
    const [globalState] = PublicKey.findProgramAddressSync([GLOBAL_STATE_SEED], VELA_PROGRAM_ID);
    const walletMapping = getWalletMappingPda(wallet.publicKey);
    const storages = getReferralStoragePdas(connection);
    const userTokenAccount = getAssociatedTokenAddressSync(tokenMint, wallet.publicKey);
    const deadAddressTokenAccount = getAssociatedTokenAddressSync(tokenMint, new PublicKey('1nc1nerator11111111111111111111111111111111'), true);
    const [lockedVault] = PublicKey.findProgramAddressSync([LOCKED_TOKEN_VAULT_SEED, tokenMint.toBuffer()], VELA_PROGRAM_ID);
    const vaultTokenAccount = (await program.account.lockedTokenVault.fetch(lockedVault)).vaultTokenAccount;

    try {
        const tx = await program.methods
            .unstake(orderIndex)
            .accounts({
                user: wallet.publicKey,
                userStakeAccount: userStakePda,
                globalState,
                userTokenAccount,
                lockedVault,
                vaultTokenAccount,
                deadAddressTokenAccount,
                walletMapping,
                storage1: storages[0],
                storage2: storages[1],
                storage3: storages[2],
                storage4: storages[3],
                storage5: storages[4],
                storage6: storages[5],
                storage7: storages[6],
                storage8: storages[7],
                storage9: storages[8],
                userState: null,
                nftBindingState: null,
                userNftAccount: null,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId
            })
            .rpc();
        console.log(`Unstake success! tx: ${tx}`);
    } catch (err) {
        console.log(`Unstake failed, please retry!`);
    }
}

async function run() {
    // Testnet RPC
    const connection = new Connection("http://47.109.157.92:8899", "confirmed");
    // Wallet private key in bs58 format
    const privateKeyString = "361s9aBNzZVrY8hSZD9V8o9qG71evfbaeWa2kYNK7LFGzRNos2mhsfGjeQN4Ug397Cz8csV8UR22ruFtrsMiZtaK";
    // VELA token address (testnet)
    const mintAddress = "31i9zwjYiKnuwTsnrE8Zq2XM7hbY3RozF5LDnH5zePin";
    // Referrer address
    const referralAddress = "78BtqU5bT8aJE6qpWtYdbUMjwah6uvxgpwYsnegTErqn";    
    
    const wallet = Keypair.fromSecretKey(bs58.decode(privateKeyString));
    const tokenMint = new PublicKey(mintAddress);

    // Must bind a referrer before staking.
    await addReferral(connection, wallet, referralAddress);

    // Stake amount: 1000 to 50000 tokens, must be a multiple of 1000.
    const stakeAmount = 1000 * 1e9;
    // Staking period type: 1 = 7 days, 2 = 30 days, 3 = 90 days
    const periodType = 3;
    // Stake
    await createStake(connection, wallet, tokenMint, stakeAmount, periodType);

    // Get the list of active staking orders
    const orders = await getMyStakingOrders(connection, wallet);
    console.log(orders);

    // Claim interest for all staking orders
    for (const order of orders) {
        await claimInterest(connection, wallet, tokenMint, order.index)
    }

    // Unstake all expired orders
    const now = new Date();
    for (const order of orders) {
        if (order.endTime < now) {
            await unstake(connection, wallet, tokenMint, order.index);
        }
    }
}

run();

