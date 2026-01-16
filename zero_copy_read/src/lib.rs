use pinocchio::{AccountView, Address, ProgramResult, entrypoint, error::ProgramError};

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    processor::process(program_id, accounts, instruction_data)
}

pub mod processor {
    use super::*; 

    pub fn process(_program_id: &Address, accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
        let discriminator = instruction_data.first().ok_or(ProgramError::InvalidInstructionData)?;
        match discriminator {
            Deposit::DISCRIMINATOR => Deposit::try_from((instruction_data, accounts))?.process(),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: InstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a[AccountView])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, account_view): (&'a[u8], &'a[AccountView])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(&account_view)?;
        let instruction_data = InstructionData::try_from(&data[1..])?;

        Ok(Self { accounts, instruction_data })
    }
}

// Small instruction data (fits in transaction)
pub struct InstructionData {
    amount: u64,
    multiplier: u64,
}

impl TryFrom<&[u8]> for InstructionData {
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != 16 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let multiplier = u64::from_le_bytes(data[8..16].try_into().unwrap());

        Ok(Self { amount, multiplier })
    }
}

// ========================================================================
// YOUR CHALLENGE: Implement deserialization for this large account data!
// ========================================================================

/// Large account data structure (5000 bytes)
/// Layout:
/// - counter: u64 (8 bytes)
/// - timestamp: i64 (8 bytes) 
/// - values: [u64; 100] (800 bytes)
/// - flags: [u8; 200] (200 bytes)
/// - data_blob: [u8; 3984] (3984 bytes)
/// Total: 5000 bytes
#[repr(C)]
pub struct UserData {
    pub counter: u64,
    pub timestamp: i64,
    pub values: [u64; 100],
    pub flags: [u8; 200],
    pub data_blob: [u8; 1984],
}

impl UserData {
    pub const SIZE: usize = 3000;
    // pub const SIZE: usize = 5000;

    // TODO: Implement field-by-field deserialization (manual copy approach)
    // This should:
    // 1. Validate data length
    // 2. Parse each field from bytes
    // 3. Copy arrays into owned arrays
    // 4. Return owned UserData
    pub fn from_bytes_manual(data: &[u8]) -> Result<Self, ProgramError> {
        // TODO: Implement this!
        // Hint: Use u64::from_le_bytes, copy_from_slice, etc.
        const BYTE_SIZE: usize = 8; 
        const EXPECTED_SIZE: usize = core::mem::size_of::<UserData>();
        const VALUES_LENGTH: usize = 800;
        const FLAGS_LENGTH: usize = 200;
        const DATA_BLOB_LENGTH: usize = 1984;
        if data.len() !=  EXPECTED_SIZE {
            return Err(ProgramError::InvalidAccountData)
        }

        let counter = u64::from_le_bytes(data[..8].try_into().unwrap());
        let timestamp = i64::from_le_bytes(data[8..16].try_into().unwrap());
        let mut values = [0u64; 100];
        let ptr_offset: usize = 16;

        for index in 0..100 {
            values[index] = u64::from_le_bytes(data[(ptr_offset + (index * BYTE_SIZE))..(ptr_offset + ((index + 1) * BYTE_SIZE))].try_into().unwrap());
        };

        let ptr_offset: usize = ptr_offset + VALUES_LENGTH;
        let flags: [u8; FLAGS_LENGTH] = data[ptr_offset..(ptr_offset + FLAGS_LENGTH)].try_into().unwrap();
        
        let ptr_offset: usize = ptr_offset + FLAGS_LENGTH;
        let data_blob: [u8; DATA_BLOB_LENGTH] = data[ptr_offset..].try_into().unwrap();

        Ok(
            Self {
                counter,
                timestamp,
                values,
                flags,
                data_blob
            }
        )

        // todo!("Implement manual deserialization")
    }

    // TODO: Implement zero-copy deserialization
    // This should:
    // 1. Validate data length
    // 2. Check alignment (optional for this challenge)
    // 3. Cast pointer to &UserData
    // 4. Return reference without copying
    pub fn from_bytes_zerocopy(data: &[u8]) -> Result<&Self, ProgramError> {
        // TODO: Implement this!
        // Hint: Use unsafe pointer casting

        if data.len() != UserData::SIZE {
            return Err(ProgramError::InvalidAccountData)
        }
        
        if (data.as_ptr() as usize) % core::mem::align_of::<UserData>() != 0 {
            return Err(ProgramError::InvalidAccountData)
        }

        Ok(unsafe { &*(data.as_ptr() as *const Self) })

        // todo!("Implement zero-copy deserialization")
    }
}

// Account structure
pub struct DepositAccounts<'a> {
    pub authority: &'a AccountView,
    pub recipient: &'a AccountView,
    pub data_account: &'a AccountView,  // This holds the large UserData!
}

impl<'a> TryFrom<&&'a[AccountView]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(account_view: &&'a[AccountView]) -> Result<Self, Self::Error> {
        let [authority, recipient, data_account, _system_program] = account_view else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // TODO: Add account validations here if needed
        // - Check authority is signer
        // - Check data_account has correct size
        // - Check ownership, etc.
        

        Ok(DepositAccounts { 
            authority, 
            recipient, 
            data_account 
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        use solana_program_log::{log_compute_units, log};

        log("Starting deposit process...");
        log_compute_units();
        
        // Option 1: Manual deserialization (copy all data)
        // let user_data = UserData::from_bytes_manual(
        //     unsafe { core::slice::from_raw_parts(
        //         self.accounts.data_account.data_ptr(),
        //         self.accounts.data_account.data_len() )
        //     }
        // )?;
        
        // Option 2: Zero-copy deserialization (no copy!)
        let user_data = UserData::from_bytes_zerocopy(
            unsafe { core::slice::from_raw_parts(
                self.accounts.data_account.data_ptr(),
                self.accounts.data_account.data_len() )
            }
        )?;

        // TODO: Once you've implemented deserialization, uncomment below:
        // /*
        // Perform computations using the deserialized data
        let mut total = 0u64;
        
        // Sum all values
        for value in user_data.values.iter() {
            total = total.wrapping_add(*value);
        }
        
        // Apply instruction data
        total = total.wrapping_mul(self.instruction_data.multiplier);
        total = total.wrapping_add(self.instruction_data.amount);
        
        // Count active flags
        let mut active_flags = 0u32;
        for flag in user_data.flags.iter() {
            if *flag > 0 {
                active_flags += 1;
            }
        }
        
        // Process data blob (just sum first 1000 bytes)
        let mut blob_sum = 0u64;
        for byte in user_data.data_blob.iter().take(1000) {
            blob_sum = blob_sum.wrapping_add(*byte as u64);
        }
        
        log(&format!("Total: {}", total));
        log(&format!("Active flags: {}", active_flags));
        log(&format!("Blob sum: {}", blob_sum));
        log(&format!("Counter: {}", user_data.counter));
        // */

        log_compute_units();
        log("Deposit complete!");

        Ok(())
    }
}