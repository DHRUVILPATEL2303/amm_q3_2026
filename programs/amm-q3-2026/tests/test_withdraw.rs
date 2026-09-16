mod common;

use common::{
    token_amount, to_address, TestEnv, INITIAL_LP, INITIAL_USER_BALANCE, INITIAL_X, INITIAL_Y,
};

#[test]
fn test_withdraw() {
    let mut env = TestEnv::initialized();
    env.deposit(INITIAL_LP, INITIAL_X, INITIAL_Y);
    env.withdraw(INITIAL_LP, 1, 1);

    assert_eq!(token_amount(&env.svm, &to_address(env.user_lp())), 0);
    assert_eq!(token_amount(&env.svm, &to_address(env.vault_x)), 0);
    assert_eq!(token_amount(&env.svm, &to_address(env.vault_y)), 0);
    assert_eq!(token_amount(&env.svm, &env.user_x), INITIAL_USER_BALANCE);
    assert_eq!(token_amount(&env.svm, &env.user_y), INITIAL_USER_BALANCE);
}
