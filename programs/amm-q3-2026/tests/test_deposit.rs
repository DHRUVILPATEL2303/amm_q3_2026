mod common;

use common::{token_amount, to_address, TestEnv, INITIAL_LP, INITIAL_X, INITIAL_Y, INITIAL_USER_BALANCE};

#[test]
fn test_deposit() {
    let mut env = TestEnv::initialized();
    env.deposit(INITIAL_LP, INITIAL_X, INITIAL_Y);

    assert_eq!(token_amount(&env.svm, &to_address(env.vault_x)), INITIAL_X);
    assert_eq!(token_amount(&env.svm, &to_address(env.vault_y)), INITIAL_Y);
    assert_eq!(
        token_amount(&env.svm, &env.user_x),
        INITIAL_USER_BALANCE - INITIAL_X
    );
    assert_eq!(
        token_amount(&env.svm, &env.user_y),
        INITIAL_USER_BALANCE - INITIAL_Y
    );
    assert_eq!(
        token_amount(&env.svm, &to_address(env.user_lp())),
        INITIAL_LP
    );
}

#[test]
fn test_deposit_rejects_zero_amount() {
    let mut env = TestEnv::initialized();
    let ix = env.deposit_ix(0, INITIAL_X, INITIAL_Y);
    assert!(env.try_send(ix).is_err());
}
