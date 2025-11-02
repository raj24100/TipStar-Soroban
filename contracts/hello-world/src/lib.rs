#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    symbol_short,
    vec, Address, Env, String, Symbol, Vec,
};

// Tip record structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tip {
    pub id: u64,
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub message: String,
    pub timestamp: u64,
}

// Statistics for a content creator
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorStats {
    pub creator: Address,
    pub total_received: i128,
    pub tip_count: u64,
    pub last_tip_timestamp: u64,
}

#[contract]
pub struct TipStar;

// Storage keys
const TIP_COUNTER: Symbol = symbol_short!("tip_cnt");
const TIPS: Symbol = symbol_short!("tips");
const CREATOR_TIPS: Symbol = symbol_short!("cr_tips");
const CREATOR_STATS: Symbol = symbol_short!("stats");
const RECENT_TIPS: Symbol = symbol_short!("recent");

const MAX_RECENT_TIPS: u32 = 100;

#[contractimpl]
impl TipStar {
    /// Send a tip to a content creator
    /// 
    /// # Arguments
    /// * `from` - Address of the sender (must be authenticated)
    /// * `to` - Address of the content creator receiving the tip
    /// * `amount` - Amount to tip (in smallest unit of the asset)
    /// * `message` - Optional message from the fan
    /// 
    /// # Returns
    /// The tip ID
    pub fn send_tip(env: Env, from: Address, to: Address, amount: i128, message: String) -> u64 {
        // Validate inputs
        from.require_auth();
        
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        // Get and increment tip counter
        let storage = env.storage().persistent();
        let tip_id: u64 = storage.get(&TIP_COUNTER).unwrap_or(0) + 1;
        storage.set(&TIP_COUNTER, &tip_id);

        // Get current timestamp (using ledger timestamp)
        let timestamp = env.ledger().timestamp();

        // Create tip record
        let tip = Tip {
            id: tip_id,
            from: from.clone(),
            to: to.clone(),
            amount,
            message,
            timestamp,
        };

        // Store tip by ID (using tuple key)
        let tip_key = (TIPS, tip_id);
        storage.set(&tip_key, &tip);

        // Store tip ID in creator's tip list
        let creator_tips_key = (CREATOR_TIPS, to.clone());
        let mut creator_tips: Vec<u64> = storage
            .get(&creator_tips_key)
            .unwrap_or_else(|| Vec::new(&env));
        creator_tips.push_back(tip_id);
        storage.set(&creator_tips_key, &creator_tips);

        // Update creator statistics
        let stats_key = (CREATOR_STATS, to.clone());
        let mut stats: CreatorStats = storage
            .get(&stats_key)
            .unwrap_or_else(|| CreatorStats {
                creator: to.clone(),
                total_received: 0,
                tip_count: 0,
                last_tip_timestamp: 0,
            });
        stats.total_received += amount;
        stats.tip_count += 1;
        stats.last_tip_timestamp = timestamp;
        storage.set(&stats_key, &stats);

        // Add to recent tips list
        let mut recent_tips: Vec<u64> = storage
            .get(&RECENT_TIPS)
            .unwrap_or_else(|| Vec::new(&env));
        recent_tips.push_back(tip_id);
        
        // Keep only the most recent N tips
        let recent_len = recent_tips.len() as u32;
        if recent_len > MAX_RECENT_TIPS {
            let start_index = recent_len - MAX_RECENT_TIPS;
            let mut trimmed = Vec::new(&env);
            for i in (start_index as usize)..recent_len as usize {
                if let Some(tip_id_val) = recent_tips.get(i as u32) {
                    trimmed.push_back(tip_id_val);
                }
            }
            recent_tips = trimmed;
        }
        storage.set(&RECENT_TIPS, &recent_tips);

        tip_id
    }

    /// Get a specific tip by ID
    pub fn get_tip(env: Env, tip_id: u64) -> Tip {
        let storage = env.storage().persistent();
        let tip_key = (TIPS, tip_id);
        storage.get(&tip_key).expect("Tip not found")
    }

    /// Get all tips for a specific creator
    /// 
    /// # Arguments
    /// * `creator` - Address of the content creator
    /// * `limit` - Maximum number of tips to return (optional, defaults to 50)
    /// * `offset` - Number of tips to skip (optional, defaults to 0)
    /// 
    /// # Returns
    /// Vector of tips
    pub fn get_creator_tips(
        env: Env,
        creator: Address,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Vec<Tip> {
        let storage = env.storage().persistent();
        let creator_tips_key = (CREATOR_TIPS, creator.clone());
        let tip_ids: Vec<u64> = storage
            .get(&creator_tips_key)
            .unwrap_or_else(|| Vec::new(&env));

        let limit = limit.unwrap_or(50).min(100); // Max 100 tips per query
        let offset = offset.unwrap_or(0);

        let tip_ids_len = tip_ids.len() as u32;
        let start = offset.min(tip_ids_len);
        let end = (start + limit).min(tip_ids_len);

        let mut tips = Vec::new(&env);

        // Iterate backwards to get most recent tips first
        for i in (start..end).rev() {
            if let Some(tip_id) = tip_ids.get(i) {
                let tip_key = (TIPS, tip_id);
                if let Some(tip) = storage.get(&tip_key) {
                    tips.push_back(tip);
                }
            }
        }

        tips
    }

    /// Get statistics for a content creator
    pub fn get_creator_stats(env: Env, creator: Address) -> CreatorStats {
        let storage = env.storage().persistent();
        let stats_key = (CREATOR_STATS, creator.clone());
        storage
            .get(&stats_key)
            .unwrap_or_else(|| CreatorStats {
                creator,
                total_received: 0,
                tip_count: 0,
                last_tip_timestamp: 0,
            })
    }

    /// Get recent tips across all creators (for widget display)
    /// 
    /// # Arguments
    /// * `limit` - Maximum number of tips to return (optional, defaults to 20, max 100)
    /// 
    /// # Returns
    /// Vector of recent tips
    pub fn get_recent_tips(env: Env, limit: Option<u32>) -> Vec<Tip> {
        let storage = env.storage().persistent();
        let tip_ids: Vec<u64> = storage
            .get(&RECENT_TIPS)
            .unwrap_or_else(|| Vec::new(&env));

        let limit = limit.unwrap_or(20).min(100); // Max 100 tips per query
        let tip_ids_len = tip_ids.len() as u32;
        let start = tip_ids_len.saturating_sub(limit);
        let end = tip_ids_len;

        let mut tips = Vec::new(&env);
        
        // Iterate backwards to get most recent tips first
        for i in (start..end).rev() {
            if let Some(tip_id) = tip_ids.get(i) {
                let tip_key = (TIPS, tip_id);
                if let Some(tip) = storage.get(&tip_key) {
                    tips.push_back(tip);
                }
            }
        }

        tips
    }

    /// Get total tip count
    pub fn get_total_tip_count(env: Env) -> u64 {
        let storage = env.storage().persistent();
        storage.get(&TIP_COUNTER).unwrap_or(0)
    }
}

mod test;
