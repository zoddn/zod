import { Keypair, PublicKey } from "@solana/web3.js";
import assert from "assert";
import * as anchor from "@project-serum/anchor";
//@ts-ignore
import idl from "../../target/idl/zod.json";
import {
  USDC_DECIMALS,
  State,
  Cluster,
  createProgram,
  createTokenAccount,
} from "@zero_one/client";
import { Idl, BN, Program } from "@project-serum/anchor";
import ZodState from "./ZodState";
import ZodMargin from "./ZodMargin";
import { withBalanceChange } from "./utils";
import { BTC_DECIMALS } from "./config";
const SETTINGS = {
  connection: "https://psytrbhymqlkfrhudd.dev.genesysgo.net:8899/",
  zoPid: new PublicKey("Zo1ThtSHMh9tZGECwBDL81WJRL6s3QTHf733Tyko7KQ"),
  zoState: new PublicKey("KwcWW7WvgSXLJcyjKZJBHLbfriErggzYHpjS9qjVD5F"),
  usdcMint: new PublicKey("7UT1javY6X1M9R2UrPGrwcZ78SX3huaXyETff5hm5YdX"),
  solMint: new PublicKey("So11111111111111111111111111111111111111112"),
  btcMint: new PublicKey("3n3sMJMnZhgNDaxp6cfywvjHLrV1s34ndfa6xAaYvpRs"),
  god: new PublicKey("2QqWUAdCfvbwVnkwSaVePnD5E5xpevZWo734JMqpY1eB"),
  godUSDC: new PublicKey("9LLF4Kv64B6YSLwryeG6jUfUgpmNRFUibLFKzWDvfCJA"),
  /*-------------------------------------------------------------------*/
  zodPid: new PublicKey("HjBqgYKdav882K1bbnoaSr3QmZ9mxQcpmAFrvrAKjrpL"),
  zodStateUSDCVault: new PublicKey(
    "HRF7zEALYDRZ6QUgRVB82LeR3fWqr8sMdoTt6B5Ymbhi"
  ),
  zodStateBTCVault: new PublicKey(
    "1XDHfhXRB5Mrg6j9gV6FDUVhPCWwv8TmJmHji3qDx2i"
  ),
  zodState: new PublicKey("GJmwLnvjpgk7ccB9REcM2QvtG29vJCzTs4om46xdFYhp"),
  zodStateMarginToZo: new PublicKey(
    "7nAQFihYpqVGWL4VJ5eXX1y5XPjmVNRpmj7ZqQ9NKG4e"
  ),
  control: new PublicKey("7d5sPfHrz42SVaphaEtUF3n469gLB2Y6qg81HXCZq1S7"),
  ZodMint: new PublicKey("62b7zbCWSjdqrGiDPLLc6o153j5mfRNp4nTgCNmAaTCa"),
  /*-------------------------------------------------------------------*/
  aliceUSDC: new PublicKey("C8WvEkicXhE46DQrq2CDx2MdPoGzfgJSwo868r8Vjr1M"),
  aliceZodTokenAcc: new PublicKey(
    "9R3H36vSgET3j4x2T4scYJC3axa5EbPBVWo8aeugM1Ci"
  ),
  /*-------------------------------------------------------------------*/
  bobUSDC: new PublicKey("2VUBou9sbzBn94XxSfHBmHGQPngGmHyerAmNpzuZ33k2"),
  bobBTC: new PublicKey("6M3avPmUx1nnzd1538kNMddp69RxZCZZ4G6vCZWT6MfW"),
  bobZodTokenAcc: new PublicKey("7bhxQwZUiYcvDp1PcVVzXTX5PZc4MdfmjJ1AswYMdN1L"),
};

function initZodState(ts: any) {
  describe("calling zod init_state.rs", async () => {
    xit("succesfully creates Zod State", async () => {
      ts.zodMint = await Keypair.generate();

      ts.state = await State.load(ts.program, SETTINGS.zoState);

      ts.zodState = await ZodState.create(ts.zodProgram, ts.program, ts.state);

      ts.zodStateUSDCVault = await createTokenAccount(
        ts.god,
        ts.usdcMint,
        ts.zodState.pubkey
      );

      console.log(ts.zodStateUSDCVault.toString());

      ts.zodStateBTCVault = await createTokenAccount(
        ts.god,
        ts.btcMint,
        ts.zodState.pubkey
      );
      console.log(ts.zodStateBTCVault.toString());

      assert.ok(ts.zodState.data.admin.equals(ts.zodState.wallet.publicKey));
    });

    xit("succesfully can have vaults", async () => {
      ts.zodProgram.rpc.addVaults({
        accounts: {
          admin: ts.god.wallet.publicKey,
          zodState: ts.zodState.publicKey,
          state: ts.zodState.pubkey,
          vault: ts.zodStateUSDCVault,
          mint: ts.usdcMint,
        },
      });
      ts.zodProgram.rpc.addVaults({
        accounts: {
          admin: ts.god.wallet.publicKey,
          zodState: ts.zodState.publicKey,
          state: ts.zodState.pubkey,
          vault: ts.zodStatebtcVault,
          mint: ts.btcMint,
        },
      });
    });
  });
}

function zodAlice(ts: any) {
  describe("calling create_margin.rs and deposit.rs and withdraw.rs", async () => {
    it("creates Alice provider and token accounts", async () => {
      ts.zodState = await ZodState.load(
        ts.zodProgram,
        ts.program,
        ts.state,
        ts.state.cache,
        SETTINGS.control,
        SETTINGS.ZodMint
      );

      const path = require("path");
      const aliceAcc = Keypair.fromSecretKey(
        Buffer.from(
          JSON.parse(
            require("fs").readFileSync(
              path.resolve(__dirname, "../alice_test.json"),
              {
                encoding: "utf-8",
              }
            )
          )
        )
      );

      const aliceProvider = new anchor.Provider(
        ts.program.provider.connection,
        // @ts-ignore
        new anchor.Wallet(aliceAcc),
        {
          preflightCommitment: "confirmed",
          commitment: "confirmed",
        }
      );
      // @ts-ignore
      ts.aliceProgram = new anchor.Program(
        ts.program.idl,
        ts.program.programId,
        aliceProvider
      );

      ts.alice = ts.aliceProgram.provider;

      ts.aliceUSDC = SETTINGS.aliceUSDC;

      // @ts-ignore
      ts.aliceZodProgram = new anchor.Program(
        ts.zodProgram.idl,
        ts.zodProgram.programId,
        ts.alice
      );
    });

    it("Alice loads zodMargin", async () => {
      ts.aliceZodMargin = await ZodMargin.load(
        ts.aliceZodProgram,
        ts.aliceProgram,
        ts.zodState,
        ts.state,
        ts.state.cache,
        SETTINGS.aliceZodTokenAcc
      );
    });

    //only if you need to recreate a new margin because you have a new alice acc
    xit("creates Alice's zodMargin account succesfully", async () => {
      ts.aliceZodMargin = await ZodMargin.create(
        ts.aliceZodProgram,
        ts.aliceProgram,
        ts.zodState,
        ts.state,
        ts.state.cache
      );
    });

    it("allows Alice to deposit", async () => {
      const depositAmount = new BN(40 * 10 ** USDC_DECIMALS);

      const [aliceUsdcChange, usdcVaultChange] = await withBalanceChange(
        ts.alice,
        [ts.aliceUSDC, ts.stateUSDCVault],
        async () => {
          const tx = await ts.aliceZodMargin.depositRaw(
            ts.aliceUSDC,
            SETTINGS.zodStateUSDCVault,
            ts.stateUSDCVault,
            ts.state.signer,
            ts.state.cache.pubkey,
            depositAmount
          );
          await ts.alice.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(usdcVaultChange, depositAmount.toNumber());
      assert.equal(aliceUsdcChange, -depositAmount.toNumber());
    });

    it("allows Alice to withdraw", async () => {
      const withdrawAmount = new BN(35 * 10 ** USDC_DECIMALS);

      const [aliceUsdcChange, usdcVaultChange] = await withBalanceChange(
        ts.alice,
        [ts.aliceUSDC, ts.stateUSDCVault],
        async () => {
          const tx = await ts.aliceZodMargin.withdrawRaw(
            ts.aliceUSDC,
            SETTINGS.zodStateUSDCVault,
            ts.stateUSDCVault,
            ts.state.signer,
            ts.state.cache.pubkey,
            ts.zodState.controlPubkey,
            withdrawAmount
          );
          await ts.alice.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(usdcVaultChange, -withdrawAmount.toNumber());
      assert.equal(aliceUsdcChange, withdrawAmount.toNumber());
    });
  });

  describe("calling mint.rs and burn.rs", async () => {
    it("allows Alice to mint funds", async () => {
      const mintAmount = new BN(9.5 * 10 ** USDC_DECIMALS);

      const [zodTokenChange] = await withBalanceChange(
        ts.alice,
        [ts.aliceZodMargin.zodTokenAccount],
        async () => {
          const tx = await ts.aliceZodMargin.mintRaw(
            ts.aliceZodMargin.zodTokenAccount,
            ts.state.cache.pubkey,
            mintAmount
          );
          await ts.alice.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(zodTokenChange, mintAmount.toNumber());
    });

    xit("stops alice from minting if they mint too much", async () => {
      const mintAmount = new BN(25 * 10 ** USDC_DECIMALS);

      const [zodTokenChange] = await withBalanceChange(
        ts.alice,
        [ts.aliceZodMargin.zodTokenAccount],
        async () => {
          try {
            const tx = await ts.aliceZodMargin.mintRaw(
              ts.aliceZodMargin.zodTokenAccount,
              ts.state.cache.pubkey,
              mintAmount
            );
            await ts.alice.connection.confirmTransaction(tx, "finalized");
            console.log("tx ", tx);
          } catch (error) {
            console.log(error);
          }
        }
      );
      assert.equal(zodTokenChange, mintAmount.toNumber());

      const [zodTokenChange1] = await withBalanceChange(
        ts.alice,
        [ts.aliceZodMargin.zodTokenAccount],
        async () => {
          try {
            const tx = await ts.aliceZodMargin.mintRaw(
              ts.aliceZodMargin.zodTokenAccount,
              ts.state.cache.pubkey,
              mintAmount
            );
            await ts.alice.connection.confirmTransaction(tx, "finalized");
            console.log("tx ", tx);
          } catch (error) {
            console.log("it should say 'assertion failed: omf > imf'");
          }
        }
      );

      assert.equal(zodTokenChange1, 0);
    });

    it("allows Alice to burn funds", async () => {
      const burnAmount = new BN(9 * 10 ** USDC_DECIMALS);

      const [zodTokenChange] = await withBalanceChange(
        ts.alice,
        [ts.aliceZodMargin.zodTokenAccount],
        async () => {
          const tx = await ts.aliceZodMargin.burnRaw(
            ts.aliceZodMargin.zodTokenAccount,
            ts.state.cache.pubkey,
            burnAmount
          );
          await ts.alice.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(zodTokenChange, -burnAmount.toNumber());
    });
  });

  describe("calling liquidate.rs", async () => {
    it("create Bob provider and token accounts", async () => {
      ts.zodState = await ZodState.load(
        ts.zodProgram,
        ts.program,
        ts.state,
        ts.state.cache,
        SETTINGS.control,
        SETTINGS.ZodMint
      );

      const path = require("path");
      const bobAcc = Keypair.fromSecretKey(
        Buffer.from(
          JSON.parse(
            require("fs").readFileSync(
              path.resolve(__dirname, "../bob_test.json"),
              {
                encoding: "utf-8",
              }
            )
          )
        )
      );

      const bobProvider = new anchor.Provider(
        ts.program.provider.connection,
        // @ts-ignore
        new anchor.Wallet(bobAcc),
        {
          preflightCommitment: "confirmed",
          commitment: "confirmed",
        }
      );
      // @ts-ignore
      ts.bobProgram = new anchor.Program(
        ts.program.idl,
        ts.program.programId,
        bobProvider
      );

      ts.bob = ts.bobProgram.provider;

      ts.bobUSDC = SETTINGS.bobUSDC;

      ts.bobBTC = SETTINGS.bobBTC;

      // @ts-ignore
      ts.bobZodProgram = new anchor.Program(
        ts.zodProgram.idl,
        ts.zodProgram.programId,
        ts.bob
      );
    });

    it("Bob loads zodMargin", async () => {
      ts.bobZodMargin = await ZodMargin.load(
        ts.bobZodProgram,
        ts.bobProgram,
        ts.zodState,
        ts.state,
        ts.state.cache,
        SETTINGS.bobZodTokenAcc
      );
    });

    xit("creates Bob's zodMargin account succesfully", async () => {
      anchor.setProvider(anchor.Provider.env());
      ts.bobZodMargin = await ZodMargin.create(
        ts.bobZodProgram,
        ts.bobProgram,
        ts.zodState,
        ts.state,
        ts.state.cache
      );
    });

    it("allows Bob to deposit btc", async () => {
      const depositAmount = new BN(1 * 10 ** BTC_DECIMALS);

      const [bobBtcChange, btcVaultChange] = await withBalanceChange(
        ts.bob,
        [ts.bobBTC, ts.stateBTCVault],
        async () => {
          const tx = await ts.bobZodMargin.depositRaw(
            ts.bobBTC,
            SETTINGS.zodStateBTCVault,
            ts.stateBTCVault,
            ts.state.signer,
            ts.state.cache.pubkey,
            depositAmount
          );
          await ts.bob.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(btcVaultChange, depositAmount.toNumber());
      assert.equal(bobBtcChange, -depositAmount.toNumber());
    });

    xit("allows Bob to mint", async () => {
      const mintAmount = new BN(10 * 10 ** USDC_DECIMALS);

      const [zodTokenChange] = await withBalanceChange(
        ts.bob,
        [ts.bobZodMargin.zodTokenAccount],
        async () => {
          const tx = await ts.bobZodMargin.mintRaw(
            ts.bobZodMargin.zodTokenAccount,
            ts.state.cache.pubkey,
            mintAmount
          );
          await ts.bob.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(zodTokenChange, mintAmount.toNumber());
    });

    xit("Bob liquidates Alice", async () => {
      const liquidateAmount = new BN(700 * 10 ** USDC_DECIMALS);

      const [zodTokenChange] = await withBalanceChange(
        ts.bob,
        [ts.bobZodMargin.zodTokenAccount],
        async () => {
          const tx = await ts.bobZodMargin.liquidate(
            ts.bobZodMargin.zodTokenAccount,
            ts.aliceZodMargin.pubkey,
            ts.usdcMint,
            ts.state.cache.pubkey,
            liquidateAmount,
            new BN(250)
          );
          await ts.bob.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(zodTokenChange, -liquidateAmount);
    });
  });
}

function bankruptcy(ts: any) {
  describe("calling add_insurance.rs, reduce_insurance.rs and settle_bankruptcy.rs", async () => {
    xit("can add to insurance fund", async () => {
      ts.zodState = await ZodState.load(
        ts.zodProgram,
        ts.program,
        ts.state,
        ts.state.cache,
        SETTINGS.control,
        SETTINGS.ZodMint
      );

      const addInsuranceAmount = new BN(1000 * 10 ** USDC_DECIMALS);

      const [godUsdcChange, usdcVaultChange] = await withBalanceChange(
        ts.god,
        [ts.godUSDC, ts.stateUSDCVault],
        async () => {
          const tx = await ts.zodState.addInsurance(
            ts.godUSDC,
            SETTINGS.zodStateUSDCVault,
            ts.stateUSDCVault,
            ts.state.signer,
            ts.state.cache.pubkey,
            addInsuranceAmount
          );
          await ts.god.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(usdcVaultChange, addInsuranceAmount.toNumber());
      assert.equal(godUsdcChange, -addInsuranceAmount.toNumber());
    });

    xit("can reduce insurance fund", async () => {
      const reduceInsuranceAmount = new BN(500 * 10 ** USDC_DECIMALS);

      const [godUsdcChange, usdcVaultChange] = await withBalanceChange(
        ts.god,
        [ts.godUSDC, ts.stateUSDCVault],
        async () => {
          const tx = await ts.zodState.reduceInsurance(
            ts.godUSDC,
            SETTINGS.zodStateUSDCVault,
            ts.stateUSDCVault,
            ts.state.signer,
            ts.state.cache.pubkey,
            ts.zodState.controlPubkey,
            reduceInsuranceAmount
          );
          await ts.god.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      assert.equal(usdcVaultChange, -reduceInsuranceAmount.toNumber());
      assert.equal(godUsdcChange, reduceInsuranceAmount.toNumber());
    });

    xit("Bob settles Alice's bankruptcy", async () => {
      const [zodTokenChange] = await withBalanceChange(
        ts.bob,
        [ts.bobZodMargin.zodTokenAccount],
        async () => {
          const tx = await ts.bobZodMargin.settleBankruptcy(
            ts.bobZodMargin.zodTokenAccount,
            ts.aliceZodMargin.pubkey,
            ts.usdcMint,
            ts.state.cache.pubkey,
            new BN(0)
          );
          await ts.bob.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      console.log("bob zod token change: ", zodTokenChange);
    });

    xit("setup for testing bankruptcy when insurance is 0", async () => {
      const reduceInsuranceAmount = new BN(10 * 10 ** USDC_DECIMALS);

      // const tx1 = await ts.zodState.reduceInsurance(
      //   ts.godUSDC,
      //   SETTINGS.zodStateUSDCVault,
      //   ts.stateUSDCVault,
      //   ts.state.signer,
      //   ts.state.cache.pubkey,
      //   ts.zodState.controlPubkey,
      //   reduceInsuranceAmount
      // );
      // await ts.god.connection.confirmTransaction(tx1, "finalized");
      // console.log("tx1 ", tx1);

      const depositAmount = new BN(100 * 10 ** USDC_DECIMALS);

      const tx2 = await ts.aliceZodMargin.depositRaw(
        ts.aliceUSDC,
        SETTINGS.zodStateUSDCVault,
        ts.stateUSDCVault,
        ts.state.signer,
        ts.state.cache.pubkey,
        depositAmount
      );
      await ts.alice.connection.confirmTransaction(tx2, "finalized");
      console.log("tx2 ", tx2);

      const mintAmount = new BN(10 * 10 ** USDC_DECIMALS);

      const tx3 = await ts.aliceZodMargin.mintRaw(
        ts.aliceZodMargin.zodTokenAccount,
        ts.state.cache.pubkey,
        mintAmount
      );
      await ts.alice.connection.confirmTransaction(tx3, "finalized");
      console.log("tx3 ", tx3);
    });

    xit("Bob settles Alice's bankruptcy when insurance is zero", async () => {
      const [zodTokenChange] = await withBalanceChange(
        ts.bob,
        [ts.bobZodMargin.zodTokenAccount],
        async () => {
          const tx = await ts.bobZodMargin.settleBankruptcy(
            ts.bobZodMargin.zodTokenAccount,
            ts.aliceZodMargin.pubkey,
            ts.usdcMint,
            ts.state.cache.pubkey,
            new BN(0)
          );
          await ts.bob.connection.confirmTransaction(tx, "finalized");
          console.log("tx ", tx);
        }
      );
      console.log("bob zod token change: ", zodTokenChange);
    });
  });
}

function fetchZoStateInfo(ts: any) {
  it("Fetching the Zo state stuff", async () => {
    const provider = anchor.Provider.local(SETTINGS.connection, {
      skipPreflight: false,
      commitment: "confirmed",
    });
    //anchor.setProvider(provider);
    ts.program = createProgram(provider, Cluster.Devnet);

    ts.state = await State.load(ts.program, SETTINGS.zoState);

    ts.god = ts.program.provider;

    console.log("admin wallet", ts.god.wallet.publicKey.toString());

    anchor.setProvider(anchor.Provider.env());

    //@ts-ignore
    ts.zodProgram = anchor.workspace.Zod as Program<Zod>;
    ts.zodProgram = new Program(idl as Idl, SETTINGS.zodPid, ts.god);

    ts.usdcMint = SETTINGS.usdcMint;
    ts.btcMint = SETTINGS.btcMint;

    ts.godUSDC = SETTINGS.godUSDC;

    ts.stateUSDCVault = ts.state.getVaultCollateralByMint(ts.usdcMint)[0];
    ts.stateBTCVault = ts.state.getVaultCollateralByMint(ts.btcMint)[0];
  });
}

describe("init zod e2e", async () => {
  const testState: any = {};

  fetchZoStateInfo(testState);

  zodAlice(testState);

  bankruptcy(testState);
});
