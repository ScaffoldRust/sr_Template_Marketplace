use soroban_sdk::{contracttype, Address, Symbol, Vec};
#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    pub id: u32,
    pub amount: i128,
    pub description: Symbol,
    pub completed: bool,
    pub approved: bool,
    pub disputed: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Transactions {
    pub buyer: Address,
    pub seller: Address,
    pub total_amount: i128,
    pub milestones: Vec<Milestone>,
    pub created_at: u64,
    pub is_active: bool,
}

#[contracttype]
pub enum DataKey {
    Transaction(u32),
    TransactionCount,
    PaymentToken,               // Stores the escrow token address
    EscrowBalance(Address),     // Tracks balance for each address
    MilestoneRelease(u32, u32), // Tracks released funds per milestone
}

pub const TIMEOUT: u64 = 604_800;
