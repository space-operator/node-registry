use solana_sdk::pubkey::Pubkey;

pub fn find_proxy_authority_address(authority: &Pubkey) -> Pubkey {
    let (expected_pda, _bump_seed) =
        Pubkey::find_program_address(&[b"proxy", &authority.to_bytes()], &space_wrapper::ID);

    /*
     * This check never fails
    let actual_pda = Pubkey::create_program_address(
        &[b"proxy", &authority.to_bytes(), &[bump_seed]],
        &Pubkey::from_str("295QjveZJsZ198fYk9FTKaJLsgAWNdXKHsM6Qkb3dsVn").unwrap(),
    )
    .unwrap();

    assert_eq!(expected_pda, actual_pda);
    */
    expected_pda
}
