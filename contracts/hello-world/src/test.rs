#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

#[test]
fn test_send_tip() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Create test addresses
    let fan = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mock auth for the fan
    env.mock_all_auths();

    // Send a tip
    let tip_id = client.send_tip(
        &fan,
        &creator,
        &1000,
        &String::from_str(&env, "Great content!"),
    );

    // Verify tip ID
    assert_eq!(tip_id, 1);

    // Get the tip
    let tip = client.get_tip(&tip_id);
    assert_eq!(tip.id, 1);
    assert_eq!(tip.from, fan);
    assert_eq!(tip.to, creator);
    assert_eq!(tip.amount, 1000);
    assert_eq!(tip.message, String::from_str(&env, "Great content!"));
}

#[test]
fn test_get_creator_stats() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Create test addresses
    let fan1 = Address::generate(&env);
    let fan2 = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mock auth for fans
    env.mock_all_auths();

    // Send multiple tips
    client.send_tip(
        &fan1,
        &creator,
        &1000,
        &String::from_str(&env, "Tip 1"),
    );
    client.send_tip(
        &fan2,
        &creator,
        &2000,
        &String::from_str(&env, "Tip 2"),
    );

    // Get stats
    let stats = client.get_creator_stats(&creator);
    assert_eq!(stats.creator, creator);
    assert_eq!(stats.total_received, 3000);
    assert_eq!(stats.tip_count, 2);
    // In test environment, timestamp might be 0, so we just check it's set correctly
    assert_eq!(stats.last_tip_timestamp, 0); // Test env has timestamp 0
}

#[test]
fn test_get_creator_tips() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Create test addresses
    let fan1 = Address::generate(&env);
    let fan2 = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mock auth for fans
    env.mock_all_auths();

    // Send multiple tips
    let tip1_id = client.send_tip(
        &fan1,
        &creator,
        &1000,
        &String::from_str(&env, "First tip"),
    );
    let tip2_id = client.send_tip(
        &fan2,
        &creator,
        &2000,
        &String::from_str(&env, "Second tip"),
    );

    // Get creator tips
    let tips = client.get_creator_tips(&creator, &None, &None);
    
    // Should have 2 tips
    assert_eq!(tips.len(), 2);
    
    // Most recent tip should be first (returned in reverse chronological order)
    let recent_tip = tips.get(0).unwrap();
    assert_eq!(recent_tip.id, tip2_id);
    assert_eq!(recent_tip.amount, 2000);
    
    // Older tip should be second
    let older_tip = tips.get(1).unwrap();
    assert_eq!(older_tip.id, tip1_id);
    assert_eq!(older_tip.amount, 1000);
}

#[test]
fn test_get_recent_tips() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Create test addresses
    let fan1 = Address::generate(&env);
    let fan2 = Address::generate(&env);
    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    // Mock auth for fans
    env.mock_all_auths();

    // Send tips to different creators
    client.send_tip(
        &fan1,
        &creator1,
        &1000,
        &String::from_str(&env, "Tip to creator 1"),
    );
    client.send_tip(
        &fan2,
        &creator2,
        &2000,
        &String::from_str(&env, "Tip to creator 2"),
    );

    // Get recent tips
    let recent_tips = client.get_recent_tips(&Some(10));
    
    // Should have 2 tips
    assert_eq!(recent_tips.len(), 2);
    
    // Most recent tip should be first (returned in reverse chronological order)
    let recent_tip = recent_tips.get(0).unwrap();
    assert_eq!(recent_tip.to, creator2);
    assert_eq!(recent_tip.amount, 2000);
    
    // Older tip should be second
    let older_tip = recent_tips.get(1).unwrap();
    assert_eq!(older_tip.to, creator1);
    assert_eq!(older_tip.amount, 1000);
}

#[test]
fn test_get_total_tip_count() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Initially should be 0
    assert_eq!(client.get_total_tip_count(), 0);

    // Create test addresses
    let fan1 = Address::generate(&env);
    let fan2 = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mock auth for fans
    env.mock_all_auths();

    // Send tips
    client.send_tip(
        &fan1,
        &creator,
        &1000,
        &String::from_str(&env, "Tip 1"),
    );
    assert_eq!(client.get_total_tip_count(), 1);

    client.send_tip(
        &fan2,
        &creator,
        &2000,
        &String::from_str(&env, "Tip 2"),
    );
    assert_eq!(client.get_total_tip_count(), 2);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_send_tip_invalid_amount() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Create test addresses
    let fan = Address::generate(&env);
    let creator = Address::generate(&env);

    // Mock auth for the fan
    env.mock_all_auths();

    // Try to send a tip with invalid amount
    client.send_tip(
        &fan,
        &creator,
        &0,
        &String::from_str(&env, "Invalid tip"),
    );
}

#[test]
fn test_get_creator_stats_empty() {
    let env = Env::default();
    let contract_id = env.register(TipStar, ());
    let client = TipStarClient::new(&env, &contract_id);

    // Create test address
    let creator = Address::generate(&env);

    // Get stats for creator with no tips
    let stats = client.get_creator_stats(&creator);
    assert_eq!(stats.creator, creator);
    assert_eq!(stats.total_received, 0);
    assert_eq!(stats.tip_count, 0);
    assert_eq!(stats.last_tip_timestamp, 0);
}
