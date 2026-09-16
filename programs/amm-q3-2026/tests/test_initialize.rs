mod common;

use common::{to_pubkey, TestEnv, FEE, SEED};

#[test]
fn test_initialize() {
    let env = TestEnv::initialized();
    let config = env.config_account();

    assert_eq!(config.seed, SEED);
    assert_eq!(config.fee, FEE);
    assert!(!config.locked);
    assert_eq!(config.mint_x, to_pubkey(env.mint_x));
    assert_eq!(config.mint_y, to_pubkey(env.mint_y));
    assert_eq!(config.authority, Some(env.payer_pk()));
    assert!(env.svm.get_account(&common::to_address(env.mint_lp)).is_some());
    assert!(env.svm.get_account(&common::to_address(env.vault_x)).is_some());
    assert!(env.svm.get_account(&common::to_address(env.vault_y)).is_some());
    assert!(env
        .svm
        .get_account(&common::to_address(env.treasury_x))
        .is_some());
    assert!(env
        .svm
        .get_account(&common::to_address(env.treasury_y))
        .is_some());
}
