import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Counter } from "../target/types/counter";
import { assert } from "chai";

describe("counter-program", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.Cal as Program<Counter>;

  // Generate a keypair for the user
  const user = anchor.web3.Keypair.generate();

  // Derive the PDA for the counter
  const [counterPDA, _bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    program.programId
  );

  // Airdrop some SOL to the user for paying fees
  before(async () => {
    await provider.connection.requestAirdrop(user.publicKey, 2e9); // 2 SOL
    await new Promise((resolve) => setTimeout(resolve, 500)); // Wait for airdrop confirmation
  });

  it("Initializes the counter to 0", async () => {
    // Call the initialize function
    await program.methods
      .initialize()
      .accounts({
        counter: counterPDA,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    // Fetch the counter account
    const counterAccount = await program.account.counter.fetch(counterPDA);

    // Assert that the counter is initialized to 0
    assert.equal(counterAccount.data, 0, "Counter should be initialized to 0");
  });

  it("Increases the counter by 1", async () => {
    // Call the increase function
    await program.methods
      .increase()
      .accounts({
        counter: counterPDA,
      })
      .signers([])
      .rpc();

    // Fetch the counter account
    const counterAccount = await program.account.counter.fetch(counterPDA);

    // Assert that the counter is increased to 1
    assert.equal(counterAccount.data, 1, "Counter should be increased to 1");
  });

  it("Increases the counter again", async () => {
    // Call the increase function again
    await program.methods
      .increase()
      .accounts({
        counter: counterPDA,
      })
      .signers([])
      .rpc();

    // Fetch the counter account
    const counterAccount = await program.account.counter.fetch(counterPDA);

    // Assert that the counter is increased to 2
    assert.equal(counterAccount.data, 2, "Counter should be increased to 2");
  });
});