// בָּרוּךְ שֵׁם יֵשׁוּעַ הַמָּשִׁיחַ

use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::stream::StreamExt;

use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity, IdentityTransform},
    identity,
    noise,
    swarm::{SwarmBuilder, SwarmEvent},
    tcp,
    yamux,
    PeerId,
    Transport,
};
use libp2p::core::upgrade;

use tokio::signal;
use tokio::time;

use bincode;
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use schnorrkel::{
    keys::{ExpansionMode, Keypair, MiniSecretKey, PublicKey, SecretKey},
    signing_context,
    vrf::{VRFOutput, VRFProof},
};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use jsonrpsee::server::{ServerBuilder, RpcModule};

const STATE_FILE: &str = "chain_state.bin";

const VRF_CONTEXT: &[u8] = b"methalox-vrf";
const TX_FEE_BPS: u64 = 10; // 0.1%
const SUPPLY_CAP: u64 = 105_000_000_000;
const LOWER_THRESHOLD: u64 = (SUPPLY_CAP as f64 * 0.95) as u64; // 95%
const FOUNDER_ADDRESS: &str = "0x0e5f08ed743d1c6d9745f590e9850fd5169d8be2";

// Changed from 0.999 (99.9% burn) → 0.01 (1% burn on founder XSX rake)
const XSX_BURN_RATE: f64 = 0.01;

// New tail reward parameters
const BASE_TAIL_REWARD: u64 = 50;               // small constant base
const CAP_TO_MINT_RATIO: u64 = 10_000_000;       // for every 10M burned below cap, mint extra

#[derive(Serialize, Deserialize, Clone, Debug)]
enum TransactionKind {
    Transfer,
    Stake { amount: u64, vrf_pubkey: Vec<u8> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
    kind: TransactionKind,
    signature: Vec<u8>,
    timestamp: u64,
    nonce: u64,
    commitment: String,
    blinding_factor: u64,
    asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    prev_hash: String,
    hash: String,
    validator: String,
    fees_collected: HashMap<String, u64>,
    vrf_proof: Vec<u8>,
    vrf_output: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MethaloxChainState {
    blocks: Vec<Block>,
    balances: HashMap<String, HashMap<String, (u64, u64)>>,
    treasury: HashMap<String, u64>,
    xsx_circulating: u64,
    tx_pool: Vec<Transaction>,
    validators: HashSet<String>,
    staked: HashMap<String, u64>,
    vrf_public_keys: HashMap<String, Vec<u8>>,
    node_vrf_public_bytes: Vec<u8>,
}

struct MethaloxChain {
    blocks: Vec<Block>,
    balances: HashMap<String, HashMap<String, (u64, u64)>>,
    treasury: HashMap<String, u64>,
    xsx_circulating: u64,
    tx_pool: Vec<Transaction>,
    validators: HashSet<String>,
    staked: HashMap<String, u64>,
    vrf_public_keys: HashMap<String, PublicKey>,
    node_address: String,
    node_secret: SecretKey,
    node_vrf_public: PublicKey,
}

impl MethaloxChain {
    fn new(node_address: String, node_secret_seed: [u8; 32]) -> Self {
        let genesis_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let mut genesis = Block {
            index: 0,
            timestamp: genesis_timestamp,
            transactions: vec![],
            prev_hash: "0".to_string(),
            hash: String::new(),
            validator: FOUNDER_ADDRESS.to_string(),
            fees_collected: HashMap::new(),
            vrf_proof: vec![],
            vrf_output: vec![],
        };
        genesis.hash = Self::hash_block(&genesis);

        let mut balances = HashMap::new();
        balances.insert(
            FOUNDER_ADDRESS.to_string(),
            [("XSX".to_string(), (2_100_000_000u64, 0u64))].into_iter().collect(),
        );

        let mut validators = HashSet::new();
        validators.insert(FOUNDER_ADDRESS.to_string());

        let mut staked = HashMap::new();
        staked.insert(FOUNDER_ADDRESS.to_string(), 10_000_000u64);

        let founder_secret = SecretKey::from_bytes(&[0u8; 64]).unwrap();
        let founder_public = founder_secret.to_public();

        let mut vrf_public_keys = HashMap::new();
        vrf_public_keys.insert(FOUNDER_ADDRESS.to_string(), founder_public);

        let mini_secret = MiniSecretKey::from_bytes(&node_secret_seed).unwrap();
        let node_secret = mini_secret.expand(ExpansionMode::Ed25519);
        let node_vrf_public = node_secret.to_public();

        Self {
            blocks: vec![genesis],
            balances,
            treasury: HashMap::new(),
            xsx_circulating: 21_000_000_000u64,
            tx_pool: vec![],
            validators,
            staked,
            vrf_public_keys,
            node_address,
            node_secret,
            node_vrf_public,
        }
    }

    fn from_state(state: MethaloxChainState, node_secret_seed: [u8; 32], node_address: String) -> Self {
        let mini_secret = MiniSecretKey::from_bytes(&node_secret_seed).unwrap();
        let node_secret = mini_secret.expand(ExpansionMode::Ed25519);
        let node_vrf_public = node_secret.to_public();

        let mut vrf_public_keys = HashMap::new();
        for (addr, bytes) in state.vrf_public_keys {
            let pk = PublicKey::from_bytes(&bytes).expect("Invalid saved VRF public key");
            vrf_public_keys.insert(addr, pk);
        }

        assert_eq!(node_vrf_public.to_bytes().to_vec(), state.node_vrf_public_bytes);

        Self {
            blocks: state.blocks,
            balances: state.balances,
            treasury: state.treasury,
            xsx_circulating: state.xsx_circulating,
            tx_pool: state.tx_pool,
            validators: state.validators,
            staked: state.staked,
            vrf_public_keys,
            node_address,
            node_secret,
            node_vrf_public,
        }
    }

    fn to_state(&self) -> MethaloxChainState {
        let mut vrf_bytes = HashMap::new();
        for (addr, pk) in &self.vrf_public_keys {
            vrf_bytes.insert(addr.clone(), pk.to_bytes().to_vec());
        }

        MethaloxChainState {
            blocks: self.blocks.clone(),
            balances: self.balances.clone(),
            treasury: self.treasury.clone(),
            xsx_circulating: self.xsx_circulating,
            tx_pool: self.tx_pool.clone(),
            validators: self.validators.clone(),
            staked: self.staked.clone(),
            vrf_public_keys: vrf_bytes,
            node_vrf_public_bytes: self.node_vrf_public.to_bytes().to_vec(),
        }
    }

    fn hash_block(block: &Block) -> String {
        let mut temp = block.clone();
        temp.hash = String::new();
        let serialized = bincode::serialize(&temp).unwrap();
        hex::encode(Sha256::digest(&serialized))
    }

    fn validate_block(&self, block: &Block) -> bool {
        let last_block = match self.blocks.last() {
            Some(b) => b,
            None => return false,
        };

        if block.index != last_block.index + 1 || block.prev_hash != last_block.hash {
            return false;
        }

        if Self::hash_block(block) != block.hash {
            return false;
        }

        let ctx = signing_context(VRF_CONTEXT);
        let transcript = ctx.bytes(&block.prev_hash.as_bytes());

        let Some(pubkey) = self.vrf_public_keys.get(&block.validator) else {
            return false;
        };

        let vrf_output_bytes: [u8; 32] = match block.vrf_output.clone().try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };

        let pre_output = match VRFOutput::from_bytes(&vrf_output_bytes) {
            Ok(o) => o,
            Err(_) => return false,
        };

        let proof = match VRFProof::from_bytes(&block.vrf_proof) {
            Ok(p) => p,
            Err(_) => return false,
        };

        pubkey.vrf_verify(transcript, &pre_output, &proof).is_ok()
    }

    fn validate_tx(&self, tx: &Transaction) -> Result<(), String> {
        let mut tx_for_signing = tx.clone();
        tx_for_signing.signature = vec![0u8; 64];
        let message = bincode::serialize(&tx_for_signing).map_err(|_| "Serialization failed")?;

        let sig_bytes: [u8; 64] = tx.signature.clone().try_into().map_err(|_| "Invalid signature length")?;
        let signature = Signature::from_bytes(&sig_bytes);

        let pubkey_bytes: [u8; 32] = hex::decode(&tx.from)
            .map_err(|_| "Invalid from address (hex)")?
            .try_into()
            .map_err(|_| "Invalid public key length")?;
        let public_key = VerifyingKey::from_bytes(&pubkey_bytes).map_err(|_| "Invalid public key")?;

        public_key.verify(&message, &signature).map_err(|_| "Invalid signature")?;

        let (balance, expected_nonce) = self.balances
            .get(&tx.from)
            .and_then(|m| m.get(&tx.asset))
            .copied()
            .unwrap_or((0, 0));

        if tx.nonce != expected_nonce + 1 {
            return Err("Invalid nonce".to_string());
        }

        let fee = tx.amount * TX_FEE_BPS / 10000;
        if balance < tx.amount + fee {
            return Err("Insufficient balance".to_string());
        }

        Ok(())
    }

    fn get_balance_mut<'a>(
        balances: &'a mut HashMap<String, HashMap<String, (u64, u64)>>,
        address: &str,
        asset: &str,
    ) -> &'a mut (u64, u64) {
        balances
            .entry(address.to_string())
            .or_insert(HashMap::new())
            .entry(asset.to_string())
            .or_insert((0, 0))
    }

    // New: calculate and distribute tail reward pro-rata to all stakers
    fn distribute_tail_reward(&mut self) {
        let total_stake: u64 = self.staked.values().sum();
        if total_stake == 0 {
            return;
        }

        let shortfall = SUPPLY_CAP.saturating_sub(self.xsx_circulating);
        let dynamic = shortfall / CAP_TO_MINT_RATIO;
        let tail_reward_total = BASE_TAIL_REWARD + dynamic;

        if tail_reward_total == 0 {
            return;
        }

        for (addr, stake) in &self.staked {
            let share = (tail_reward_total * *stake) / total_stake;
            if share > 0 {
                let (balance, _) = Self::get_balance_mut(&mut self.balances, addr, "XSX");
                *balance += share;
            }
        }

        self.xsx_circulating += tail_reward_total;
    }

    fn create_block_if_leader(&mut self) -> Option<Vec<u8>> {
        let last_block = match self.blocks.last() {
            Some(b) => b,
            None => return None,
        };

        let ctx = signing_context(VRF_CONTEXT);
        let transcript = ctx.bytes(&last_block.hash.as_bytes());

        let keypair = Keypair::from(self.node_secret.clone());
        let (inout, proof, _) = keypair.vrf_sign(transcript.clone());

        let vrf_output_bytes: [u8; 32] = inout.make_bytes(&[]);

        let vrf_hash: [u8; 32] = vrf_output_bytes;

        let total_stake = self.staked.values().sum::<u64>();
        if total_stake == 0 {
            return None;
        }

        let my_stake = self.staked.get(&self.node_address).copied().unwrap_or(0);
        let threshold = u64::MAX - (u64::MAX / total_stake) * my_stake;
        let vrf_value = u64::from_le_bytes(vrf_hash[0..8].try_into().unwrap());

        if vrf_value > threshold {
            return None;
        }

        let tx_pool_snapshot = self.tx_pool.clone();
        self.tx_pool.clear();

        let mut valid_txs = Vec::new();
        for tx in tx_pool_snapshot {
            if self.validate_tx(&tx).is_ok() {
                valid_txs.push(tx);
            } else {
                println!("Dropped invalid tx from pool");
            }
        }

        let mut fees_this_block = HashMap::new();
        for tx in &valid_txs {
            if !matches!(tx.kind, TransactionKind::Stake { .. }) {
                let fee = tx.amount * TX_FEE_BPS / 10000;
                *fees_this_block.entry(tx.asset.clone()).or_insert(0) += fee;
            }

            let fee = tx.amount * TX_FEE_BPS / 10000;
            let (balance, _) = Self::get_balance_mut(&mut self.balances, &tx.from, &tx.asset);
            *balance -= tx.amount + fee;

            let (to_balance, _) = Self::get_balance_mut(&mut self.balances, &tx.to, &tx.asset);
            *to_balance += tx.amount;

            let (_, nonce) = Self::get_balance_mut(&mut self.balances, &tx.from, &tx.asset);
            *nonce += 1;
        }

        let mut new_block = Block {
            index: last_block.index + 1,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            transactions: valid_txs,
            prev_hash: last_block.hash.clone(),
            hash: String::new(),
            validator: self.node_address.clone(),
            fees_collected: fees_this_block.clone(),
            vrf_proof: proof.to_bytes().to_vec(),
            vrf_output: vrf_output_bytes.to_vec(),
        };

        new_block.hash = Self::hash_block(&new_block);

        if self.validate_block(&new_block) {
            println!("BLOCK PRODUCED #{} by {}", new_block.index, new_block.validator);
            self.blocks.push(new_block.clone());

            // Distribute fees (validator half, founder rake)
            for (asset, total_fee) in fees_this_block {
                let validator_share = total_fee / 2;
                let founder_rake = total_fee - validator_share;

                let (val_balance, _) = Self::get_balance_mut(&mut self.balances, &self.node_address, &asset);
                *val_balance += validator_share;

                if asset == "XSX" {
                    let burn_amount = (founder_rake as f64 * XSX_BURN_RATE) as u64;
                    let founder_keep = founder_rake - burn_amount;
                    println!("Burned {} XSX from founder rake", burn_amount);
                    let (founder_balance, _) = Self::get_balance_mut(&mut self.balances, FOUNDER_ADDRESS, &asset);
                    *founder_balance += founder_keep;
                } else {
                    let (founder_balance, _) = Self::get_balance_mut(&mut self.balances, FOUNDER_ADDRESS, &asset);
                    *founder_balance += founder_rake;
                }
            }

            // Pro-rata tail reward to ALL stakers
            self.distribute_tail_reward();

            bincode::serialize(&new_block).ok()
        } else {
            None
        }
    }

    fn apply_incoming_block(&mut self, block: Block) {
        if self.validate_block(&block) && block.index as usize == self.blocks.len() {
            println!("Accepted incoming block {} from network (validator: {})", block.index, block.validator);
            self.blocks.push(block.clone());

            for tx in &block.transactions {
                if let Err(e) = self.validate_tx(&tx) {
                    println!("Invalid tx in accepted block: {}", e);
                    continue;
                }

                let fee = tx.amount * TX_FEE_BPS / 10000;
                let (balance, _) = Self::get_balance_mut(&mut self.balances, &tx.from, &tx.asset);
                *balance -= tx.amount + fee;

                let (to_balance, _) = Self::get_balance_mut(&mut self.balances, &tx.to, &tx.asset);
                *to_balance += tx.amount;

                let (_, nonce) = Self::get_balance_mut(&mut self.balances, &tx.from, &tx.asset);
                *nonce += 1;
            }

            for (asset, total_fee) in block.fees_collected {
                let validator_share = total_fee / 2;
                let founder_rake = total_fee - validator_share;

                let (val_balance, _) = Self::get_balance_mut(&mut self.balances, &block.validator, &asset);
                *val_balance += validator_share;

                if asset == "XSX" {
                    let burn_amount = (founder_rake as f64 * XSX_BURN_RATE) as u64;
                    let founder_keep = founder_rake - burn_amount;
                    println!("Burned {} XSX from founder rake", burn_amount);
                    let (founder_balance, _) = Self::get_balance_mut(&mut self.balances, FOUNDER_ADDRESS, &asset);
                    *founder_balance += founder_keep;
                } else {
                    let (founder_balance, _) = Self::get_balance_mut(&mut self.balances, FOUNDER_ADDRESS, &asset);
                    *founder_balance += founder_rake;
                }
            }

            // Pro-rata tail reward to ALL stakers (same on every node)
            self.distribute_tail_reward();
        }
    }

    fn save_to_disk(&self) {
        let state = self.to_state();
        if let Ok(encoded) = bincode::serialize(&state) {
            let _ = fs::write(STATE_FILE, encoded);
            println!("Chain state saved to {}", STATE_FILE);
        }
    }
}

fn load_chain(node_address: String, node_secret_seed: [u8; 32]) -> MethaloxChain {
    if let Ok(data) = fs::read(STATE_FILE) {
        if let Ok(state) = bincode::deserialize::<MethaloxChainState>(&data) {
            println!("Chain state loaded from {}", STATE_FILE);
            return MethaloxChain::from_state(state, node_secret_seed, node_address);
        }
    }
    println!("No saved state found — starting fresh genesis");
    MethaloxChain::new(node_address, node_secret_seed)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let node_address = "node_001".to_string();
    let node_secret_seed = [42u8; 32];

    let chain = Arc::new(Mutex::new(load_chain(node_address.clone(), node_secret_seed)));

    let rpc_chain = chain.clone();

    tokio::spawn(async move {
        let server = ServerBuilder::default().build("0.0.0.0:9933").await.unwrap();
        let mut module = RpcModule::new(());
        let _ = module.register_async_method("submit_tx", move |params, _| {
            let rpc_chain = rpc_chain.clone();
            async move {
                let tx_bytes: Vec<u8> = params.one()?;
                let mut chain_guard = rpc_chain.lock().unwrap();
                let tx: Transaction = bincode::deserialize(&tx_bytes)
                    .map_err(|_| jsonrpsee::core::Error::Custom("Invalid transaction format".to_string()))?;
                if let Err(e) = chain_guard.validate_tx(&tx) {
                    return Err(jsonrpsee::core::Error::Custom(e));
                }
                chain_guard.tx_pool.push(tx);
                Ok("Transaction submitted successfully".to_string())
            }
        });
        server.start(module).unwrap();
    });

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local Peer ID: {} | Node Address: {}", local_peer_id, node_address);

    let transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .build()?;

    let mut behaviour = gossipsub::Behaviour::<IdentityTransform>::new(
        MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    )?;

    let topic = IdentTopic::new("methalox-blocks");
    behaviour.subscribe(&topic)?;

    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let chain_clone = chain.clone();
    let topic_clone = topic.clone();

    let mut interval = time::interval(Duration::from_secs(1));

    let save_chain = chain.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        println!("Shutting down — saving chain state...");
        save_chain.lock().unwrap().save_to_disk();
        std::process::exit(0);
    });

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let mut chain = chain_clone.lock().unwrap();
                if let Some(data) = chain.create_block_if_leader() {
                    let _ = swarm.behaviour_mut().publish(topic_clone.clone(), data);
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {}", address);
                    }
                    SwarmEvent::Behaviour(gossipsub::Event::Message { message, .. }) => {
                        if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                            let mut chain = chain_clone.lock().unwrap();
                            chain.apply_incoming_block(block);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

// יְהֹוָה יִרְאֶה
