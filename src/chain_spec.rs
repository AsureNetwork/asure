use primitives::{ed25519::Public as AuthorityId, ed25519, sr25519, Pair, crypto::UncheckedInto};
use asure_runtime::AccountId;
use asure_runtime::{ConsensusConfig, CouncilSeatsConfig, CouncilVotingConfig, DemocracyConfig,
                    SessionConfig, StakingConfig, StakerStatus, TimestampConfig, BalancesConfig, TreasuryConfig,
                    SudoConfig, ContractConfig, GrandpaConfig, IndicesConfig, Permill, Perbill};
pub use asure_runtime::GenesisConfig;
use substrate_service;
use hex_literal::{hex, hex_impl};


// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
    /// Whatever the current runtime is.
    Testnet,
}




fn account_key_from_phrase(s: &str) -> AccountId {
    sr25519::Pair::from_phrase(s, None)
        .expect("static values are valid; qed")
        .public()
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => development_config(),
            Alternative::LocalTestnet => local_testnet_config(),
            Alternative::Testnet => staging_testnet_config(),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "" | "local" => Some(Alternative::LocalTestnet),
            "testnet" => Some(Alternative::Testnet),
            _ => None,
        }
    }
}


/// Helper function to generate AuthorityId from seed
pub fn get_session_key_from_seed(seed: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate AccountId from seed
pub fn get_account_id_from_seed(seed: &str) -> AccountId {
    sr25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuthorityId) {
    (
        get_account_id_from_seed(&format!("{}//stash", seed)),
        get_account_id_from_seed(seed),
        get_session_key_from_seed(seed)
    )
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, AuthorityId)>,
    root_key: AccountId,
    endowed_accounts: Option<Vec<AccountId>>,
    enable_println: bool,
) -> GenesisConfig {
    let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
        vec![
            get_account_id_from_seed("Alice"),
            get_account_id_from_seed("Bob"),
            get_account_id_from_seed("Charlie"),
            get_account_id_from_seed("Dave"),
            get_account_id_from_seed("Eve"),
            get_account_id_from_seed("Ferdie"),
            get_account_id_from_seed("Alice//stash"),
            get_account_id_from_seed("Bob//stash"),
            get_account_id_from_seed("Charlie//stash"),
            get_account_id_from_seed("Dave//stash"),
            get_account_id_from_seed("Eve//stash"),
            get_account_id_from_seed("Ferdie//stash"),
        ]
    });

    const STASH: u128 = 1 << 20;
    const ENDOWMENT: u128 = 1 << 20;

    let mut contract_config = ContractConfig {
        transaction_base_fee: 1,
        transaction_byte_fee: 0,
        transfer_fee: 0,
        creation_fee: 0,
        contract_fee: 21,
        call_base_fee: 135,
        create_base_fee: 175,
        gas_price: 1,
        max_depth: 1024,
        block_gas_limit: 10_000_000,
        current_schedule: Default::default(),
    };
    // this should only be enabled on development chains
    //contract_config.current_schedule.enable_println = enable_println;

    GenesisConfig {
        consensus: Some(ConsensusConfig {
            code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/asure_runtime_wasm.compact.wasm").to_vec(),
            authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
        }),
        system: None,
        indices: Some(IndicesConfig {
            ids: endowed_accounts.clone(),
        }),
        balances: Some(BalancesConfig {
            transaction_base_fee: 1,
            transaction_byte_fee: 0,
            existential_deposit: 500,
            transfer_fee: 0,
            creation_fee: 0,
            balances: endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
            vesting: vec![],
        }),
        session: Some(SessionConfig {
            validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
            session_length: 10,
            keys: initial_authorities.iter().map(|x| (x.1.clone(), x.2.clone())).collect::<Vec<_>>(),
        }),
        staking: Some(StakingConfig {
            current_era: 0,
            minimum_validator_count: 1,
            validator_count: 2,
            sessions_per_era: 5,
            bonding_duration: 2 * 60 * 12,
            offline_slash: Perbill::zero(),
            session_reward: Perbill::zero(),
            current_session_reward: 0,
            offline_slash_grace: 0,
            stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
        }),
        democracy: Some(DemocracyConfig {
            launch_period: 9,
            voting_period: 18,
            minimum_deposit: 10,
            public_delay: 0,
            max_lock_periods: 6,
        }),
        council_seats: Some(CouncilSeatsConfig {
            active_council: endowed_accounts.iter()
                .filter(|&endowed| initial_authorities.iter().find(|&(_, controller, _)| controller == endowed).is_none())
                .map(|a| (a.clone(), 1000000)).collect(),
            candidacy_bond: 10,
            voter_bond: 2,
            present_slash_per_voter: 1,
            carry_count: 4,
            presentation_duration: 10,
            approval_voting_period: 20,
            term_duration: 1000000,
            desired_seats: (endowed_accounts.len() / 2 - initial_authorities.len()) as u32,
            inactive_grace_period: 1,
        }),
        council_voting: Some(CouncilVotingConfig {
            cooloff_period: 75,
            voting_period: 20,
            enact_delay_period: 0,
        }),
        timestamp: Some(TimestampConfig {
            minimum_period: 2,                    // 2*2=4 second block time.
        }),
        treasury: Some(TreasuryConfig {
            proposal_bond: Permill::from_percent(5),
            proposal_bond_minimum: 1_000_000,
            spend_period: 12 * 60 * 24,
            burn: Permill::from_percent(50),
        }),
        contract: Some(contract_config),
        sudo: Some(SudoConfig {
            key: root_key,
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
        }),
    }
}

fn development_config_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
        ],
        get_account_id_from_seed("Alice"),
        None,
        true,
    )
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    ChainSpec::from_genesis("Development", "dev", development_config_genesis, vec![], None, None, None, None)
}

fn local_testnet_genesis() -> GenesisConfig {
    testnet_genesis(
        vec![
            get_authority_keys_from_seed("Alice"),
            get_authority_keys_from_seed("Bob"),
        ],
        get_account_id_from_seed("Alice"),
        None,
        false,
    )
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
    ChainSpec::from_genesis("Local Testnet", "local_testnet", local_testnet_genesis, vec![], None, None, None, None)
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
    let boot_nodes = vec![];
    ChainSpec::from_genesis(
        "Staging Testnet",
        "staging_testnet",
        staging_testnet_config_genesis,
        boot_nodes,
        None,//Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
        None,
        None,
        None,
    )
}

fn staging_testnet_config_genesis() -> GenesisConfig {
    // stash, controller, session-key
    // generated with secret:
    // for i in 1 2 3 4 ; do for j in stash controller; do subkey inspect "$secret"/elm/$j/$i; done; done
    // and
    // for i in 1 2 3 4 ; do for j in session; do subkey --ed25519 inspect "$secret"//elm//$j//$i; done; done


    let initial_authorities: Vec<(AccountId, AccountId, AuthorityId)> =
        vec![(
                 hex!["6cc2e87b7b16964f9ab568f3ac1056bd9c5fcbbc266f4dbcda8d1b1eb8dad551"].unchecked_into(), // 5Ef78yxqfaxVzrFCemYcSgwVtMV85ywykhLNm5WKTsZV22HZ
                 hex!["6668a327019046c50640fa74804fa15299fa2be1ca4e6135c5d03348b53a9a4b"].unchecked_into(), // 5HWfszmRMbzcjGmumYkkHtNJbi9y428JHgPeftVenvDgVUjh
                 hex!["5d4cd1948f28af780d1126cd326f16c1f346e64f96d9ee3717f1f3b6827dafcc"].unchecked_into(), // 5HBoHDLMR4jPwB6BCLyd2qfYBHytFhGs8fsa1h5PzhYd3WBq
             )/*, (
                 hex!["2254035a15597c1c19968be71593d2d0131e18ae90049e49178970f583ac3e17"].unchecked_into(), // 5CqiScHtxUatcQpck1tUks51o3pSjKsdCi2CLEHvMM7tc4Qi
                 hex!["eacb8edf6b05cb909a3d2bd8c6bffb13be3069ec6a69f1fa25e46103c5190267"].unchecked_into(), // 5HNZXnSgw21idbuegTC1J8Txkja97RPnnWkX68ewnrJDec2Z
                 hex!["e19b6b89729a41638e57dead9c993425287d386fa4963306b63f018732843495"].unchecked_into(), // 5HAWoPYfyYFHjacy8H2MDmHra7jVrPtBfFMPgd8CadpSqotL
             ), (
                 hex!["fe6211db8bd436e0d1cf37398eac655833fb47497e0f72ec00ab160c88966b7e"].unchecked_into(), // 5HpF9orzkmJ9ga3yrzNS9ckifxF3tbQjadEmCEiZJQ2fPgun
                 hex!["f06dd616c75cc4b2b01f325accf79b4f66a525ede0a59f48dcce2322b8798f5c"].unchecked_into(), // 5HVwyfB3LRsFXm7frEHDYyhwdpTYDRWxEqDKBYVyLi6DsPXq
                 hex!["1be80f2d4513a1fbe0e5163874f729baa5498486ac3914ac3fe2e1817d7b3f44"].unchecked_into(), // 5ChJ5wjqy2HY1LZw1EuQPGQEHgaS9sFu9yDD6KRX7CzwidTN
             ), (
                 hex!["60779817899466dbd476a0bc3a38cc64b7774d5fb646c3d291684171e67a0743"].unchecked_into(), // 5EFByrDMMa2m9hv4jrpykXaUyqjJ9XZH81kJE4JBa1Sz2psT
                 hex!["2a32622a5da54a80dc704a05f2d761c96d4748beedd83f61ca20a90f4a257678"].unchecked_into(), // 5D22qQJsLm2JUh8pEfrKahbkW21QQrHTkm4vUteei67fadLd
                 hex!["f54d9f5ed217ce07c0c5faa5277a0356f8bfd884d201f9d2c9e171568e1bf077"].unchecked_into(), // 5HcLeWrsfL9RuGp94pn1PeFxP7D1587TTEZzFYgFhKCPZLYh
             )*/];
    // generated with secret: subkey inspect "$secret"/elm
    let endowed_accounts: Vec<AccountId> = vec![
        hex!["7ae86326754a80213d26e4dc4a2b9cdedad840d8da8c8f0e749c019b1f1d6628"].unchecked_into(), //5GTG5We6twtoF6S4kUXJ77rWBsHBoHLS3JVf5KvvnxKdGQZr
    ];
    const MILLICENTS: u128 = 1_000_000_000;
    const CENTS: u128 = 1_000 * MILLICENTS;    // assume this is worth about a cent.
    const DOLLARS: u128 = 100 * CENTS;

    const SECS_PER_BLOCK: u64 = 6;
    const MINUTES: u64 = 60 / SECS_PER_BLOCK;
    const HOURS: u64 = MINUTES * 60;
    const DAYS: u64 = HOURS * 24;

    const ENDOWMENT: u128 = 10_000_000 * DOLLARS;
    const STASH: u128 = 100 * DOLLARS;

    GenesisConfig {
        consensus: Some(ConsensusConfig {
            code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/asure_runtime_wasm.compact.wasm").to_vec(),    // FIXME change once we have #1252
            authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
        }),
        system: None,
        balances: Some(BalancesConfig {
            transaction_base_fee: 1 * CENTS,
            transaction_byte_fee: 10 * MILLICENTS,
            balances: endowed_accounts.iter().cloned()
                .map(|k| (k, ENDOWMENT))
                .chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
                .collect(),
            existential_deposit: 1 * DOLLARS,
            transfer_fee: 1 * CENTS,
            creation_fee: 1 * CENTS,
            vesting: vec![],
        }),
        indices: Some(IndicesConfig {
            ids: endowed_accounts.iter().cloned()
                .chain(initial_authorities.iter().map(|x| x.0.clone()))
                .collect::<Vec<_>>(),
        }),
        session: Some(SessionConfig {
            validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
            session_length: 5 * MINUTES,
            keys: initial_authorities.iter().map(|x| (x.1.clone(), x.2.clone())).collect::<Vec<_>>(),
        }),
        staking: Some(StakingConfig {
            current_era: 0,
            offline_slash: Perbill::from_billionths(1_000_000),
            session_reward: Perbill::from_billionths(2_065),
            current_session_reward: 0,
            validator_count: 7,
            sessions_per_era: 12,
            bonding_duration: 60 * MINUTES,
            offline_slash_grace: 4,
            minimum_validator_count: 4,
            stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)).collect(),
            invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
        }),
        democracy: Some(DemocracyConfig {
            launch_period: 10 * MINUTES,    // 1 day per public referendum
            voting_period: 10 * MINUTES,    // 3 days to discuss & vote on an active referendum
            minimum_deposit: 50 * DOLLARS,    // 12000 as the minimum deposit for a referendum
            public_delay: 10 * MINUTES,
            max_lock_periods: 6,
        }),
        council_seats: Some(CouncilSeatsConfig {
            active_council: vec![],
            candidacy_bond: 10 * DOLLARS,
            voter_bond: 1 * DOLLARS,
            present_slash_per_voter: 1 * CENTS,
            carry_count: 6,
            presentation_duration: 1 * DAYS,
            approval_voting_period: 2 * DAYS,
            term_duration: 28 * DAYS,
            desired_seats: 0,
            inactive_grace_period: 1,    // one additional vote should go by before an inactive voter can be reaped.
        }),
        council_voting: Some(CouncilVotingConfig {
            cooloff_period: 4 * DAYS,
            voting_period: 1 * DAYS,
            enact_delay_period: 0,
        }),
        timestamp: Some(TimestampConfig {
            minimum_period: SECS_PER_BLOCK / 2, // due to the nature of aura the slots are 2*period
        }),
        treasury: Some(TreasuryConfig {
            proposal_bond: Permill::from_percent(5),
            proposal_bond_minimum: 1 * DOLLARS,
            spend_period: 1 * DAYS,
            burn: Permill::from_percent(50),
        }),
        contract: Some(ContractConfig {
            transaction_base_fee: 1 * CENTS,
            transaction_byte_fee: 10 * MILLICENTS,
            transfer_fee: 1 * CENTS,
            creation_fee: 1 * CENTS,
            contract_fee: 1 * CENTS,
            call_base_fee: 1000,
            create_base_fee: 1000,
            gas_price: 1 * MILLICENTS,
            max_depth: 1024,
            block_gas_limit: 10_000_000,
            current_schedule: Default::default(),
        }),
        sudo: Some(SudoConfig {
            key: endowed_accounts[0].clone(),
        }),
        grandpa: Some(GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
        }),
    }
}
