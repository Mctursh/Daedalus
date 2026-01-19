use pinocchio::{
    AccountView, Address, ProgramResult, address::declare_id, cpi::Signer, entrypoint, error::ProgramError, instruction::seeds, sysvars::{Sysvar, rent::Rent}
};
use pinocchio_system::{instructions::{CreateAccount, Transfer}};

declare_id!("GaU5s1X1UswZhC9RwSJncPMx3NP97kszcqHjFMxJHBwK");

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let discriminator = instruction_data
        .first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => batch_transfer_direct(accounts, instruction_data),
        1 => batch_transfer_cpi(accounts, instruction_data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn batch_transfer_direct(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    use solana_program_log::{log_compute_units, log};

    log("Starting DIRECT batch transfer...");
    log_compute_units();

    // Parse instruction data
    // Format: [discriminator(1)] + [amount_per_account(8)]
    if instruction_data.len() != 20 {
    // if instruction_data.len() != 19 {
    // if instruction_data.len() != 9 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
    // let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
    log(&format!("Amount per account: {}", amount));

    let address_bumps: [u8; 11] = instruction_data[9..20].try_into().unwrap(); 

    // Parse accounts
    // Expected: [source(writable), dest1(writable), dest2(writable), ..., dest10(writable)]
    if accounts.len() != 14 {
    // if accounts.len() != 11 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let source = &accounts[0]; // account funding PDA
    let destinations = &accounts[1..11]; // 10 destination accounts
    // let destinations = &accounts[..11]; // 11 destination accounts including source
    let funding_account = &accounts[11]; // PDA for direct lampot debit
    let payer = &accounts[12]; // 10 destination accounts

    // Source account creation
    if funding_account.lamports() == 0 {
    // if funding_account.lamports() == 0 && funding_account.owned_by(&ID) {
        let lamports = Rent::get()?.minimum_balance_unchecked(0); // Zero because of no data needed
        let seed = format!("dest-10");
        let bump = &[address_bumps[10]];
        let seeds = seeds!(seed.as_bytes(), bump);
        let signer = Signer::from(&seeds);
        CreateAccount {
            from: payer,
            to: funding_account,
            lamports,
            owner: &ID,
            space: 0
        }.invoke_signed(&[signer])?;

        
        // Funding the funding account that will be doing the distribution.
        Transfer {
            from: source,
            to: funding_account,
            lamports: 2_000_000_000
        }.invoke()?;
    }


    // Validate source account is writable and has enough lamports
    if !source.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    
    if !payer.is_signer() {
        return Err(ProgramError::InvalidAccountData);
    }
    
    let total_amount = amount.checked_mul(10).ok_or(ProgramError::ArithmeticOverflow)?;
    if funding_account.lamports() < total_amount {
        return Err(ProgramError::InsufficientFunds);
    }
    
    for (index, dest) in destinations.iter().enumerate() {

        // account creation status including 
        if dest.lamports() == 0 {
        // if dest.lamports() == 0 && dest.owned_by(&ID) {
            let lamports = Rent::get()?.minimum_balance_unchecked(0); // Zero because of no data needed
            let seed = format!("dest-{}", index);
            let bump = &[address_bumps[index]];
            let seeds = seeds!(seed.as_bytes(), bump);
            let signer = Signer::from(&seeds);
            CreateAccount {
                from: payer,
                to: dest,
                lamports,
                owner: &ID,
                space: 0
            }.invoke_signed(&[signer])?;
        }

        if !dest.is_writable() {
            return Err(ProgramError::AccountBorrowFailed)
        }

        dest.set_lamports(dest.lamports() + amount);
        funding_account.set_lamports(funding_account.lamports() - amount);
    }

    log_compute_units();
    log("Direct transfer complete!");

    Ok(())
}


fn batch_transfer_cpi(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    use solana_program_log::{log_compute_units, log};

    log("Starting CPI batch transfer...");
    log_compute_units();

    // Parse instruction data
    if instruction_data.len() != 9 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
    log(&format!("Amount per account: {}", amount));

    // Parse accounts
    // Expected: [source(signer+writable), dest1(writable), ..., dest10(writable), system_program]
    if accounts.len() != 12 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let source = &accounts[0];
    let destinations = &accounts[1..11];
    let _system_program = &accounts[11];

    // Validate source is signer
    if !source.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Your code here:
    for dest in destinations {
        Transfer {
            from: source,
            to: dest,
            lamports: amount
        }.invoke()?;
    }

    log_compute_units();
    log("CPI transfer complete!");

    Ok(())
}