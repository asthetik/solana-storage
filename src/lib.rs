use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use solana_system_interface::instruction;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account_user = next_account_info(accounts_iter)?;
    if !account_user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    let account_data = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;

    let rent_exemption = Rent::get()?.minimum_balance(instruction_data.len());

    let (_pda, bump_seed) = Pubkey::find_program_address(&[account_user.key.as_ref()], program_id);

    if **account_data.try_borrow_lamports()? == 0 {
        let creat_idx = instruction::create_account(
            account_user.key,
            account_data.key,
            rent_exemption,
            instruction_data.len() as u64,
            program_id,
        );
        invoke_signed(
            &creat_idx,
            accounts,
            &[&[account_user.key.as_ref(), &[bump_seed]]],
        )?;

        account_data
            .data
            .borrow_mut()
            .copy_from_slice(instruction_data);
    }

    if rent_exemption > account_data.lamports() {
        invoke(
            &instruction::transfer(
                account_user.key,
                account_data.key,
                rent_exemption - account_data.lamports(),
            ),
            accounts,
        )?;
    } else if rent_exemption < account_data.lamports() {
        let diff = account_data.lamports() - rent_exemption;
        **account_user.try_borrow_mut_lamports()? += diff;
        **account_data.try_borrow_mut_lamports()? -= diff;
    }

    account_data.resize(instruction_data.len())?;
    account_data
        .data
        .borrow_mut()
        .copy_from_slice(instruction_data);

    Ok(())
}
