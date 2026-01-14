const {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    sendAndConfirmTransaction,
  } = require('@solana/web3.js');
  
  async function main() {
    // Connect to local validator
    const connection = new Connection('http://localhost:8899', 'confirmed');
    
    // Your deployed program ID (you'll get this after deployment)
    const programId = new PublicKey('JE7SG89kwa6ryb24fUrsLLJDGBxHqMfxH7tMhJZuMxHf');
    
    // Create a payer (use test validator's default keypair)
    const payer = Keypair.generate();
    
    // Airdrop SOL to payer
    console.log('Airdropping SOL to payer...');
    const airdropSignature = await connection.requestAirdrop(
      payer.publicKey,
      2_000_000_000 // 2 SOL
    );
    await connection.confirmTransaction(airdropSignature);
    console.log('Airdrop confirmed!');
    
    // Create instruction with discriminator 0
    const instruction = new TransactionInstruction({
      keys: [], // No accounts needed for log_checker
      programId,
      data: Buffer.from([0]), // discriminator 0
    });
    
    // Create and send transaction
    console.log('Sending transaction...');
    const transaction = new Transaction().add(instruction);
    
    try {
      const signature = await sendAndConfirmTransaction(
        connection,
        transaction,
        [payer],
        {
          commitment: 'confirmed',
        }
      );
      
      console.log('✅ Transaction successful!');
      console.log('Signature:', signature);
      
      // Fetch logs to see your print! output
      const logs = await connection.getTransaction(signature, {
        commitment: 'confirmed',
        maxSupportedTransactionVersion: 0
      });
      console.log('\nProgram logs:');
      console.log(logs.meta.logMessages.join('\n'));
      
    } catch (error) {
      console.error('❌ Transaction failed:', error);
      throw error;
    }
  }
  
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });