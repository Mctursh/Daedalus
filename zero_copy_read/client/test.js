const {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    sendAndConfirmTransaction,
    SystemProgram,
  } = require('@solana/web3.js');
  
  async function main() {
    const connection = new Connection('http://localhost:8899', 'confirmed');
    const programId = new PublicKey('Bivb1bzUR3CKtpwKAww8CdyLViKnMFJn1EPFvrfa2mic');
    
    // Create accounts
    const payer = Keypair.generate();
    const authority = Keypair.generate();
    const recipient = Keypair.generate();
    const dataAccount = Keypair.generate();
    
    // Fund payer
    console.log('💰 Funding payer...');
    const airdrop = await connection.requestAirdrop(payer.publicKey, 5_000_000_000);
    await connection.confirmTransaction(airdrop);
    
    // Create data account
    console.log('📝 Creating data account...');
    const DATA_SIZE = 3000;
    // const DATA_SIZE = 5000;
    const rent = await connection.getMinimumBalanceForRentExemption(DATA_SIZE);
    
    const createAccount = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: dataAccount.publicKey,
      lamports: rent,
      space: DATA_SIZE,
      programId: SystemProgram.programId,
    });
    
    await sendAndConfirmTransaction(
      connection,
      new Transaction().add(createAccount),
      [payer, dataAccount]
    );
    
    // Build test data (5000 bytes)
    const counter = Buffer.alloc(8);
    counter.writeBigUInt64LE(42n);
    
    const timestamp = Buffer.alloc(8);
    timestamp.writeBigInt64LE(BigInt(Date.now()));
    
    const values = Buffer.alloc(800);
    for (let i = 0; i < 100; i++) {
      values.writeBigUInt64LE(BigInt(i * 1000), i * 8);
    }
    
    const flags = Buffer.alloc(200);
    for (let i = 0; i < 200; i++) {
      flags[i] = i % 2;
    }
    
    const dataBlob = Buffer.alloc(1984);
    // const dataBlob = Buffer.alloc(3984);
    for (let i = 0; i < 1984; i++) {
      dataBlob[i] = i % 256;
    }
    
    const fullData = Buffer.concat([counter, timestamp, values, flags, dataBlob]);
    
    // Write data to account (localnet only!)
    console.log('✍️  Writing test data...');
    await connection._rpcRequest('setAccount', [
      dataAccount.publicKey.toBase58(),
      {
        lamports: rent,
        data: Array.from(fullData),
        owner: SystemProgram.programId.toBase58(),
        executable: false,
      }
    ]);
    
    console.log('✅ Account ready with test data!\n');
    
    // Now test your program
    console.log('📤 Testing your program...');
    
    const discriminator = Buffer.from([0]);
    const amount = Buffer.alloc(8);
    amount.writeBigUInt64LE(3000n);
    // amount.writeBigUInt64LE(5000n);
    const multiplier = Buffer.alloc(8);
    multiplier.writeBigUInt64LE(3n);
    
    const ix = new TransactionInstruction({
      keys: [
        { pubkey: authority.publicKey, isSigner: true, isWritable: false },
        { pubkey: recipient.publicKey, isSigner: false, isWritable: true },
        { pubkey: dataAccount.publicKey, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId,
      data: Buffer.concat([discriminator, amount, multiplier]),
    });
    
    const sig = await sendAndConfirmTransaction(
      connection,
      new Transaction().add(ix),
      [payer, authority]
    );
    
    console.log('✅ Success!');
    console.log('Signature:', sig, '\n');
    
    // Show logs
    const tx = await connection.getTransaction(sig, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0
    });
    
    console.log('📊 Logs:');
    tx.meta.logMessages.forEach(log => console.log(log));
  }
  
  main().catch(console.error);