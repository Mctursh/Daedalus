use pinocchio::{AccountView, Address, ProgramResult, entrypoint, error::ProgramError};

use solana_program_log::{log, log_compute_units};



#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);


pub fn process_instruction(program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    processor::process(program_id, accounts, instruction_data)
}

pub mod processor {
    use super::*; 

    pub fn process(program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
        let discriminator= instruction_data.first().ok_or(ProgramError::InvalidInstructionData)?;
        match discriminator   {
            0 => instruction::log_checker(program_id, accounts, instruction_data),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}

mod instruction {

    use super::*;

    pub fn log_checker(_program_id: &Address, _accounts: &[AccountView], _instruction_data: &[u8]) -> ProgramResult {
        log_compute_units();
        
        for i in 0..10 {
            log(&format!("The square of the current index {i} is equal to {}", i * i));
        }
    
        log_compute_units();
    
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use pinocchio::Address;

    use crate::{instruction};

    #[test]
    fn test_log_checker() {
        let program_id = Address::from([0u8; 32]);
        let accounts = vec![];
        let instruction_data = vec![0u8];

        let result = instruction::log_checker(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok());

    }
}