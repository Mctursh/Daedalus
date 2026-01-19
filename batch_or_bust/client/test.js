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
    const programId = new PublicKey('GaU5s1X1UswZhC9RwSJncPMx3NP97kszcqHjFMxJHBwK');
    
    // Create accounts
    const payer = Keypair.generate();
    const sourceAccount = Keypair.generate();
    
    // Create 10 destination accounts
    const destinations = Array.from({ length: 10 }, () => Keypair.generate());
    const pdaSeeds = Array.from({ length: 11 }, (_, i) => `dest-${i}`); // 10 plus 1 the source
    // const pdaSeeds = Array.from({ length: 10 }, (_, i) => `dest-${i}`);
    const pdaAddresses = []
    const bumpPerAddress = []
    
    console.log('💰 Setting up accounts...\n');
    
    // Fund payer
    const airdrop = await connection.requestAirdrop(payer.publicKey, 10_000_000_000);
    await connection.confirmTransaction(airdrop);
    console.log('✅ Payer funded');
    
    // Fund source account with enough for transfers
    const fundSource = await connection.requestAirdrop(sourceAccount.publicKey, 5_000_000_000);
    await connection.confirmTransaction(fundSource);
    console.log('✅ Source account funded');

    // const [sourcePda, sourceBump] = PublicKey.findProgramAddressSync(
    //     [Buffer.from("vault")],
    //     programId
    //   );
    //   pdaAddresses.push(sourcePda);
    //   bumpPerAddress.push(sourceBump)

    

    
    // Create all destination accounts (they need to exist)
    console.log('📝 Creating destination accounts...');
    for (let i = 0; i < pdaSeeds.length; i++) {
    //   const rent = await connection.getMinimumBalanceForRentExemption(0);
      const [pda, bump] = PublicKey.findProgramAddressSync(
        [
            Buffer.from(pdaSeeds[i])
        ],
        programId
      )

      pdaAddresses.push(pda);
      bumpPerAddress.push(bump)
      
    //   const tx = new Transaction().add(createIx);
    //   await sendAndConfirmTransaction(connection, tx, [payer, destinations[i]]);
      console.log(`  ✓ PDA ${i + 1} created`);
    }
    
    console.log('\n🎯 Testing DIRECT lamport transfer (cheap)...\n');
    
    // Get balances before
    const sourceBalanceBefore = await connection.getBalance(sourceAccount.publicKey);
    // const destBalancesBefore = await Promise.all(
    //   pdaAddresses.map(d => connection.getBalance(d.publicKey))
    // //   destinations.map(d => connection.getBalance(d.publicKey))
    // );
    
    // Build instruction for direct transfer
    const discriminator = Buffer.from([0]); // 0 = direct
    const amountPerAccount = 1_000_000n; // 0.001 SOL per account
    const amountBuffer = Buffer.alloc(8);
    amountBuffer.writeBigUInt64LE(amountPerAccount);
    
    const directInstruction = new TransactionInstruction({
      keys: [
        // { pubkey: sourcePda, isSigner: false, isWritable: true },
        { pubkey: sourceAccount.publicKey, isSigner: true, isWritable: true },
        ...pdaAddresses.map(d => ({
        // ...destinations.map(d => ({
          pubkey: d,
          isSigner: false,
          isWritable: true,
        })),
        // payer
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId,
      data: Buffer.concat([discriminator, amountBuffer, Buffer.from(bumpPerAddress)]),
    });
    
    const directTx = new Transaction().add(directInstruction);
    const directSig = await sendAndConfirmTransaction(
      connection,
      directTx,
      [payer, sourceAccount],
      { commitment: 'confirmed' }
    );
    
    console.log('✅ Direct transfer successful!');
    console.log(`Signature: ${directSig}\n`);
    
    // Get transaction details
    const directTxDetails = await connection.getTransaction(directSig, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });
    
    console.log('📊 Direct Transfer Logs:');
    directTxDetails.meta.logMessages.forEach(log => console.log(log));
    
    const directCU = directTxDetails.meta.logMessages.find(log =>
      log.includes('consumed') && log.includes('compute units')
    );
    if (directCU) {
      console.log(`\n⚡ ${directCU}\n`);
    }
    
    // Verify balances
    const sourceBalanceAfter = await connection.getBalance(sourceAccount.publicKey);
    const destBalancesAfter = await Promise.all(
      destinations.map(d => connection.getBalance(d.publicKey))
    );
    
    console.log('💸 Balance Changes:');
    console.log(`Source: ${sourceBalanceBefore} → ${sourceBalanceAfter} (${sourceBalanceBefore - sourceBalanceAfter} lamports sent)`);
    // for (let i = 0; i < 10; i++) {
    //   console.log(`  Dest ${i + 1}: ${destBalancesBefore[i]} → ${destBalancesAfter[i]} (+${destBalancesAfter[i] - destBalancesBefore[i]})`);
    // }
    
    console.log('\n' + '='.repeat(70));
    console.log('\n🎯 Testing CPI transfer (expensive)...\n');
    
    // Reset: fund source again
    const refund = await connection.requestAirdrop(sourceAccount.publicKey, 1_000_000_000);
    await connection.confirmTransaction(refund);



    // ==================================== MID ========================================
    
    // Build instruction for CPI transfer
    // const cpiDiscriminator = Buffer.from([1]); // 1 = CPI
    // const cpiInstruction = new TransactionInstruction({
    //   keys: [
    //     { pubkey: sourceAccount.publicKey, isSigner: true, isWritable: true }, // Must be signer for CPI
    //     ...destinations.map(d => ({
    //       pubkey: d.publicKey,
    //       isSigner: false,
    //       isWritable: true,
    //     })),
    //     { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    //   ],
    //   programId,
    //   data: Buffer.concat([cpiDiscriminator, amountBuffer]),
    // });
    
    // const cpiTx = new Transaction().add(cpiInstruction);
    // const cpiSig = await sendAndConfirmTransaction(
    //   connection,
    //   cpiTx,
    //   [payer, sourceAccount], // sourceAccount must sign!
    //   { commitment: 'confirmed' }
    // );
    
    // console.log('✅ CPI transfer successful!');
    // console.log(`Signature: ${cpiSig}\n`);
    
    // // Get transaction details
    // const cpiTxDetails = await connection.getTransaction(cpiSig, {
    //   commitment: 'confirmed',
    //   maxSupportedTransactionVersion: 0,
    // });
    
    // console.log('📊 CPI Transfer Logs:');
    // cpiTxDetails.meta.logMessages.forEach(log => console.log(log));
    
    // const cpiCU = cpiTxDetails.meta.logMessages.find(log =>
    //   log.includes('consumed') && log.includes('compute units')
    // );
    // if (cpiCU) {
    //   console.log(`\n⚡ ${cpiCU}\n`);
    // }
    
    console.log('\n' + '='.repeat(70));
    console.log('\n📈 COMPARISON:');
    console.log('Direct lamport manipulation: Should be < 2,000 CU');
    console.log('CPI to System Program: Should be 15,000-25,000 CU');
    console.log('\n💡 Key Insight: Direct lamport manipulation is ~10-20x cheaper!');
  }
  
  main()
    .then(() => {
      console.log('\n🎉 Challenge complete!');
      process.exit(0);
    })
    .catch((error) => {
      console.error('\n💥 Error:', error);
      process.exit(1);
    });