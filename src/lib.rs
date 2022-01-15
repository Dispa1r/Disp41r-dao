use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, ext_contract, near_bindgen, Promise, AccountId, PromiseResult};

use std::{collections::HashMap, collections::HashSet,io::StderrLock};
use std::fs::read_to_string;
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
near_sdk::setup_alloc!();
#[ext_contract(ext_ft)]

pub trait FungibleToken {
    fn ft_balance_of(&mut self, account_id: AccountId) -> U128;
    fn airDrop(&mut self,account : AccountId,receiveId:AccountId,num : u128) -> String;
    fn get_tmp_winner(&self) -> String;
    fn register(&mut self,account : AccountId) -> String;
}
// define methods we'll use as callbacks on our contract
#[ext_contract(ext_self)]
pub trait MyContract {
    fn my_callback(&mut self,candidate:String) -> String;
    fn vote_callback(&mut self,name :String) -> String;
    fn my_callback3(&self) ->String;
    fn my_callback1(&self) ->String;
    fn start_callback1(&mut self,title:String,time:u32,starter:String) ->String;
}
const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;


// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
// define the methods we'll use on the other contract

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Voting {
    votes_received: HashMap<String, i32>,
    //aha : LookupMap<String,i32>
    voters : HashSet<String>,
    voter_result : HashMap<String, String>,
    voter_balance : HashMap<String, u128>,
    sum_pool : u128,
    is_lock : bool,
    winner : String,
    title : String,
    durance : u64,
    starter : String,
    token_balance: HashMap<String, i32>,
}

#[near_bindgen]
impl Voting {
    #[init]
    pub fn new(title : String,time : u32,starter : String) -> Self {
        let mut nowTime = env::block_timestamp();
        nowTime += time as u64 * 1000000000 *60;
        Self {
            votes_received: HashMap::new(),
            voters : HashSet::new(),
            voter_result :HashMap::new(),
            voter_balance :HashMap::new(),
            token_balance : HashMap::new(),
            is_lock:false,
            winner:"who will be the winner?".to_string(),
            title : title,
            durance : nowTime,
            sum_pool : 0,
            starter,
        }
    }

    pub fn check_out_of_time(&mut self) -> bool{
        if env::block_timestamp() > self.durance{
            self.is_lock = true;
            let mut max = -1;
            let mut name= "";
            for (candidate, votes) in self.votes_received.iter() {
                if votes > &max {
                    max = *votes;
                    name = candidate;
                }
            }
            self.winner = name.to_string();
            self.transfer_winner();
            return true;
        }
        return false;
    }

    pub fn get_title(&self) ->String{
        return self.title.to_string();
    }

    pub fn add_candidate(&mut self, candidate: String) -> Promise{
        if self.valid_candidate(&candidate) {
            return Promise::new("the candidate has been added".to_string());
        }
        ext_ft::ft_balance_of(
            env::predecessor_account_id(),
            &"disp1.testnet", // contract account id
            0, // yocto NEAR to attach
            5_000_000_000_000 // gas to attach
        )
            // After the smart contract method finishes a DataReceipt will be sent back
            // .then registers a method to handle that incoming DataReceipt
            .then(ext_self::my_callback(
                candidate,
                &env::current_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                5_000_000_000_000_0 // gas to attach to the callback
            ))
    }

    pub fn get_total_votes_for(self, name: String) -> Option::<i32> {
        if !self.valid_candidate(&name) {
            ()
        }
        self.votes_received.get(&name).cloned()
    }

    pub fn vote(&mut self, name: String) -> String{
        if self.check_out_of_time(){
            return "the voting is over, please use get_winner function to check!".to_string();
        }
        let mut z2 = String::new();
        z2.push_str(&name);
        if !self.valid_candidate(&name) {
            return "before vote, please add the candidate".to_string();
        }
        let result = self.voters.insert(env::predecessor_account_id());
        if !result{
            return "you have voted, anyone only have one time".to_string();
        }
        if self.is_lock {
            return "the voting is over, please use get_winner function to check!".to_string();
        }else {
            ext_ft::ft_balance_of(
                env::predecessor_account_id(),
                &"disp1.testnet", // contract account id
                0, // yocto NEAR to attach
                5_000_000_000_000 // gas to attach
            )
                // After the smart contract method finishes a DataReceipt will be sent back
                // .then registers a method to handle that incoming DataReceipt
                .then(ext_self::vote_callback(
                    name,
                    &env::current_account_id(), // this contract's account id
                    0, // yocto NEAR to attach to the callback
                    5_000_000_000_000_0 // gas to attach to the callback
                ));
            return "go and check the leaderborad!".to_string();
        }

    }

    pub fn transfer_winner(&self){
        let mut winnerVec:HashSet<String> = HashSet::new();
        if self.is_lock{
            for (voter, candidate) in self.voter_result.iter() {
                if candidate.eq(&self.winner){
                    winnerVec.insert(voter.to_string());
                }
            }
        }
        let mut eachNum = self.sum_pool / (winnerVec.len() as u128);
        for (voter) in winnerVec.iter(){
            Promise::new(voter.to_string()).transfer(eachNum);
        }
    }

    pub fn get_tmp_winner(&self) -> String{
        let mut max = -1;
        let mut name= "";
        for (candidate, votes) in self.votes_received.iter() {
            if votes > &max {
                max = *votes;
                name = candidate;
            }
        }
        return name.to_string();
    }

    pub fn check_my_vote(&mut self) -> String{
        let mut id = env::predecessor_account_id();
        return self.voter_result.get(&*id).unwrap().clone();
    }

    pub fn lock(&mut self){
        assert_eq!(
            env::predecessor_account_id(),
            self.starter,
            "only the voting starter could end the voting"
        );
        self.is_lock = true;
        let mut max = -1;
        let mut name= "";
        for (candidate, votes) in self.votes_received.iter() {
            if votes > &max {
                max = *votes;
                name = candidate;
            }
        }
        self.winner = name.to_string();
        let mut tmp = String::new();
        tmp.push_str(&*self.winner);
        self.winnerAirDrop(tmp);
        self.transfer_winner();
    }

    pub fn winnerAirDrop(&self, account_id: String) -> Promise {
        // Invoke a method on another contract
        // This will send an ActionReceipt to the shard where the contract lives.
        let mut z2 = String::new();
        z2.push_str(&account_id);
        ext_ft::airDrop(
            (&"disp1.testnet").to_string(),
            account_id.into(),
            1000 as u128,
            &"disp1.testnet", // contract account id
            0, // yocto NEAR to attach
            100_000_000_000_000 // gas to attach
        )
            .then(ext_self::my_callback1(
                &env::current_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                100_000_000_000_000// gas to attach to the callback
            ))
    }

    pub fn valid_candidate(&self, name: &String) -> bool {
        for (candidate, votes) in self.votes_received.iter() {
            if self.votes_received.contains_key(name) {
                return true
            }
        }
        false
    }

    #[payable]
    pub fn register(&mut self) -> Promise{
        ext_ft::register(
            env::predecessor_account_id(),
            &"disp1.testnet", // contract account id
            env::attached_deposit(), // yocto NEAR to attach
            100_000_000_000_000 // gas to attach
        )
            .then(ext_self::my_callback1(
                &env::current_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                100_000_000_000_000// gas to attach to the callback
            ))
    }

    pub fn restart(&mut self,title:String,time:u32,starter:String) -> Promise{
        ext_ft::ft_balance_of(
            env::predecessor_account_id(),
            &"disp1.testnet", // contract account id
            0, // yocto NEAR to attach
            5_000_000_000_000 // gas to attach
        )
            // After the smart contract method finishes a DataReceipt will be sent back
            // .then registers a method to handle that incoming DataReceipt
            .then(ext_self::start_callback1(
                title,
                time,
                starter,
                &env::current_account_id(), // this contract's account id
                0, // yocto NEAR to attach to the callback
                5_000_000_000_000_0 // gas to attach to the callback
            ))
    }

    pub fn get_candidates(&self) -> HashMap<String, i32> {
        self.votes_received.clone()
    }
    pub fn get_voters(&self) -> HashSet<String> {
        self.voters.clone()
    }

    pub fn my_callback(&mut self,candidate : String) -> String {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                if balance.0 > 3 {
                    if self.check_out_of_time(){
                        return "the voting is over, please use get_winner function to check!".to_string();
                    }
                    if self.is_lock {
                        return "the voting is over, please use get_winner function to check!".to_string();
                    }
                    self.votes_received.insert(candidate, 0);
                    return "Success to add the candidate!".to_string();
                } else {
                    "you are not a memeber of dao, please hold  at least 10 DISP token".to_string()
                }
            },
        }
    }

    pub fn start_callback1(&mut self,title:String,time:u32,starter:String) ->String {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                if balance.0 > 5 {
                    assert!(
                        self.is_lock,
                        "only after the voting end you can start a new one"
                    );
                    let mut nowTime = env::block_timestamp();
                    nowTime += time as u64 * 1000000000 *60;
                    self.votes_received=HashMap::new();
                    self.voters = HashSet::new();
                    self.voter_result =HashMap::new();
                    self.voter_balance =HashMap::new();
                    self.is_lock=false;
                    self.winner="who will be the winner?".to_string();
                    self.title = title;
                    self.durance = nowTime;
                    self.sum_pool =0;
                    self.starter = starter;
                    return "Success to start a new voting!".to_string();
                } else {
                    "you are not a memeber of dao, please hold  at least 10 DISP token".to_string()
                }
            },
        }
    }


    pub fn vote_callback(&mut self,name : String) -> String {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );
        let mut z1 = String::new();
        z1.push_str(&*name);

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                if balance.0 > 3 {
                    let counter = self.votes_received.entry(name).or_insert(0);
                    *counter += ((balance.0 as i32)-3) * 10 +1;
                    self.voter_result.insert(env::predecessor_account_id(),z1.to_string());
                    return "success to vote!".to_string();
                } else {
                    "you are not a memeber of dao, please hold at least 10 DISP token".to_string()
                }
            },
        }
    }

    pub fn my_callback1(&self) -> String {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );

        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => "oops!".to_string(),
            PromiseResult::Successful(result) => {
                let suc = near_sdk::serde_json::from_slice::<String>(&result).unwrap();
                suc
            },
        }
    }
}

// #[cfg(not(target_arch = "wasm32"))]
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use near_sdk::VMContext;
//
//     fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
//         VMContext {
//             current_account_id: "alice_near".to_string(),
//             signer_account_id: "bob_near".to_string(),
//             signer_account_pk: vec![0, 1, 2],
//             predecessor_account_id: "carol_near".to_string(),
//             input,
//             block_index: 0,
//             block_timestamp: 0,
//             epoch_height: 0,
//             account_balance: 0,
//             account_locked_balance: 0,
//             storage_usage: 0,
//             attached_deposit: 0,
//             prepaid_gas: 10u64.pow(18),
//             random_seed: vec![0, 1, 2],
//             is_view,
//             output_data_receivers: vec![],
//         }
//     }
//
//     #[test]
//     fn test_add_candidate() {
//         let context = get_context(vec![], false);
//         testing_env!(context);
//         let mut contract = Voting::new("aha".to_string(), 100);
//         contract.add_candidate("Jeff".to_string());
//         assert_eq!(0, contract.get_total_votes_for("Jeff".to_string()).unwrap());
//     }
//
//     #[test]
//     fn test_get_total_votes_for() {
//         let context = get_context(vec![], true);
//         testing_env!(context);
//         let contract = Voting::new();
//         assert_eq!(None, contract.get_total_votes_for("Anna".to_string()));
//     }
// }
