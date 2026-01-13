
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &solana_program::pubkey::Pubkey,
    _accounts: &[solana_program::account_info::AccountInfo],
    _instruction_data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    solana_program::msg!("Hello, Solana Storage Program!");
    Ok(())
}