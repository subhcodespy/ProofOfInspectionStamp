#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Env, Address, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Session {
    pub id: u64,
    pub tutor: Address,
    pub student: Address,
    pub timestamp: u64,
    pub duration_minutes: u32,
    pub confirmed: bool,
    pub paid: bool,
}

#[contracttype]
pub enum SessionKey {
    Count,
    Record(u64),
}

#[contracttype]
pub enum BalanceKey {
    Bal(Address),
}

#[contract]
pub struct TutorSessionSeal;

#[contractimpl]
impl TutorSessionSeal {
    // Record a tutoring session. Caller is usually an offchain relayer or tutor
    pub fn record_session(
        env: Env,
        tutor: Address,
        student: Address,
        duration_minutes: u32,
    ) -> u64 {
        let mut count: u64 = env.storage().instance().get(&SessionKey::Count).unwrap_or(0);
        count = count.saturating_add(1);
        env.storage().instance().set(&SessionKey::Count, &count);

        let s = Session {
            id: count,
            tutor: tutor.clone(),
            student: student.clone(),
            timestamp: env.ledger().timestamp(),
            duration_minutes,
            confirmed: false,
            paid: false,
        };
        env.storage().instance().set(&SessionKey::Record(count), &s);
        count
    }

    // Student confirms the session; this makes it eligible for payment
    pub fn confirm_session(env: Env, session_id: u64, caller: Address) {
        let mut s: Session =
            env.storage().instance().get(&SessionKey::Record(session_id)).expect("session not found");
        assert!(!s.confirmed, "already confirmed");
        assert!(caller == s.student, "only student can confirm");
        s.confirmed = true;
        env.storage().instance().set(&SessionKey::Record(session_id), &s);
    }

    // Release a payout representation to the tutor when confirmed. In production connect to token transfer
    pub fn pay_session(env: Env, session_id: u64, rate_per_minute: u128) {
        let mut s: Session =
            env.storage().instance().get(&SessionKey::Record(session_id)).expect("session not found");
        assert!(s.confirmed, "session not confirmed");
        assert!(!s.paid, "already paid");

        s.paid = true;
        env.storage().instance().set(&SessionKey::Record(session_id), &s);

        let payout = (s.duration_minutes as u128).saturating_mul(rate_per_minute);
        let bal_key = BalanceKey::Bal(s.tutor.clone());
        let cur: u128 = env.storage().instance().get(&bal_key).unwrap_or(0u128);
        env.storage().instance().set(&bal_key, &(cur.saturating_add(payout)));
    }

    // Tutor withdraws internal balance
    pub fn withdraw(env: Env, who: Address) -> u128 {
        let key = BalanceKey::Bal(who);
        let cur: u128 = env.storage().instance().get(&key).unwrap_or(0u128);
        env.storage().instance().set(&key, &0u128);
        cur
    }

    // view session
    pub fn view_session(env: Env, id: u64) -> Session {
        env.storage().instance().get(&SessionKey::Record(id)).expect("session not found")
    }
}
