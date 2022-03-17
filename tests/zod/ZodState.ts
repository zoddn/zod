const anchor = require("@project-serum/anchor");
import { BN, Program } from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {
  Commitment,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
import { State, Zo, Cache, CONTROL_ACCOUNT_SIZE } from "@zero_one/client";
import { Zod } from "../../target/types/zod";
import { ZodBaseAccount } from "./ZodBaseAccount";
import { ZodStateSchema } from "./zodTypes";

const MINT_ACCOUNT_SIZE = 82
 /**
   * The ZodState account is a PDA generated using
   * ```javascript
   * seeds=[userWalletKey, stateKey, "zodv1"]
   * ```.
   */
export default class ZodState extends ZodBaseAccount<ZodStateSchema> {
  private constructor(
    zodProgram: Program<Zod>,
    zoProgram: Program<Zo>,
    pubkey: PublicKey,
    data: ZodStateSchema,
    public readonly state: State,
    public readonly controlPubkey: PublicKey,
    public readonly mint: PublicKey,
  ) {
    super(zodProgram, zoProgram, pubkey, data);
  }

  static async load(
    zodProgram: Program<Zod>,
    zoProgram: Program<Zo>,
    st: State,
    ch: Cache,
    controlPubkey: PublicKey,
    mint: PublicKey,
  ): Promise<ZodState> {
    const [key, _nonce] = await PublicKey.findProgramAddress(
      [anchor.utils.bytes.utf8.encode("zodv12")],
      zodProgram.programId);

    const data = await this.fetch(zodProgram, key, st, ch);
    return new this(zodProgram, zoProgram, key, {...data}, st, controlPubkey, mint);
  } 

  static async create(
    zodProgram: Program<Zod>,
    zoProgram: Program<Zo>,
    zoState: State,
    commitment: Commitment = "finalized",
  ): Promise<ZodState> {

    const conn = zodProgram.provider.connection;

    const [
    [zodStateKey, zodStateNonce], 
    control,
    mint, 
    controlLamports,
    mintLamports] 
    = await Promise.all([
      PublicKey.findProgramAddress(
        [anchor.utils.bytes.utf8.encode("zodv12")],
        zodProgram.programId),
        Keypair.generate(),
        Keypair.generate(),
        conn.getMinimumBalanceForRentExemption(CONTROL_ACCOUNT_SIZE),
        conn.getMinimumBalanceForRentExemption(MINT_ACCOUNT_SIZE),
      ]);

    const [zoMarginKey, zoMarginNonce] = await PublicKey.findProgramAddress(
      [zodStateKey.toBuffer(), zoState.pubkey.toBuffer(), 
        anchor.utils.bytes.utf8.encode("marginv1")],
      zoProgram.programId);

    const tx = await zodProgram.rpc.initZodState(zodStateNonce, zoMarginNonce, 
      {accounts: {
        admin: zodProgram.provider.wallet.publicKey,
        zodState: zodStateKey,
        zoProgramState: zoState.pubkey,
        zoProgramMargin: zoMarginKey,
        zoProgram: zoProgram.programId,
        control: control.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        zoProgramMarginRent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        mint: mint.publicKey,
      },
      preInstructions: [
        SystemProgram.createAccount({
          fromPubkey: zoProgram.provider.wallet.publicKey,
          newAccountPubkey: control.publicKey,
          lamports: controlLamports,
          space: CONTROL_ACCOUNT_SIZE,
          programId: zoProgram.programId,
        }),
        SystemProgram.createAccount({
          fromPubkey: zodProgram.provider.wallet.publicKey,
          newAccountPubkey: mint.publicKey,
          lamports: mintLamports,
          space: MINT_ACCOUNT_SIZE,
          programId: TOKEN_PROGRAM_ID,
        })
      ],
      signers: [control, mint],
    })

    console.log("tx:",tx);

    await conn.confirmTransaction( 
      tx,
      commitment,
    );
    return await ZodState.load(zodProgram, zoProgram, zoState, zoState.cache, control.publicKey, mint.publicKey);
  }

  private static async fetch(
    program: Program<Zod>,
    k: PublicKey,
    st: State,
    ch: Cache,
  ): Promise<ZodStateSchema> {
    const data = (await program.account["zodState"].fetch(k,"recent")) as ZodStateSchema;

    return {...data}
  }

  async addInsurance(
    tokenAccount: PublicKey,
    zodVault: PublicKey,
    zoVault: PublicKey,
    stateSigner: PublicKey,
    cache: PublicKey,
    amount: BN,
  ) {
    return await this.zodProgram.rpc.zodAddInsurance(amount, {
      accounts: {
        zodState: this.pubkey,
        zoProgramMargin: this.data.zoProgramMargin,
        zoProgram: this.program.programId,
        zoProgramState: this.data.zoProgramState,
        stateSigner: stateSigner,
        cache: cache,
        authority: this.wallet.publicKey,
        tokenAccount: tokenAccount,
        zoVault: zoVault,
        zodVault: zodVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
  }

  async reduceInsurance(
    tokenAccount: PublicKey,
    zodVault: PublicKey,
    zoVault: PublicKey,
    stateSigner: PublicKey,
    cache: PublicKey,
    control: PublicKey,
    amount: BN,
  ) {
    return await this.zodProgram.rpc.zodReduceInsurance(amount, {
      accounts: {
        zoProgramMargin: this.data.zoProgramMargin,
        zoProgram: this.program.programId,
        zodState: this.pubkey,
        zoProgramState: this.data.zoProgramState,
        stateSigner: stateSigner,
        cache: cache,
        admin: this.wallet.publicKey,
        control: control,
        tokenAccount: tokenAccount,
        zoVault: zoVault,
        zodVault: zodVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
  }

  /**
   * Refreshes the data on the ZodState, state, cache and control accounts.
   */
  async refresh(): Promise<void> {
    [this.data] = await Promise.all([
      ZodState.fetch(this.zodProgram, this.pubkey, this.state, this.state.cache),
      this.state.refresh(),
    ]);
  }

}