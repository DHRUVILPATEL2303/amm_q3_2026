mod common;

use common::{token_amount, to_address, TestEnv, INITIAL_LP, INITIAL_X, INITIAL_Y};

#[test]
fn test_swap() {
    let mut env = TestEnv::initialized();
    env.deposit(INITIAL_LP, INITIAL_X, INITIAL_Y);

    let user_x_before = token_amount(&env.svm, &env.user_x);
    let user_y_before = token_amount(&env.svm, &env.user_y);
    let vault_x_before = token_amount(&env.svm, &to_address(env.vault_x));
    let vault_y_before = token_amount(&env.svm, &to_address(env.vault_y));
    let treasury_x_before = token_amount(&env.svm, &to_address(env.treasury_x));

    let amount_in = 1_000_000;
    env.swap(true, amount_in, 1);

    let user_x_after = token_amount(&env.svm, &env.user_x);
    let user_y_after = token_amount(&env.svm, &env.user_y);
    let vault_x_after = token_amount(&env.svm, &to_address(env.vault_x));
    let vault_y_after = token_amount(&env.svm, &to_address(env.vault_y));
    let treasury_x_after = token_amount(&env.svm, &to_address(env.treasury_x));

    assert_eq!(user_x_before - user_x_after, amount_in);
    assert!(user_y_after > user_y_before);
    assert!(vault_x_after > vault_x_before);
    assert!(vault_y_after < vault_y_before);
    assert!(treasury_x_after > treasury_x_before);
    assert_eq!(
        (user_x_before - user_x_after),
        (vault_x_after - vault_x_before) + (treasury_x_after - treasury_x_before)
    );
}
