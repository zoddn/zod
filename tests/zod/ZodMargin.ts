const anchor = require("@project-serum/anchor");
import { BN, Program } from "@project-serum/anchor";
import { TOKEN_PROGRAM_ID, AccountLayout} from "@solana/spl-token";
import {
  Commitment,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  CONTROL_ACCOUNT_SIZE,
  loadWI80F48,
  Num,
  State,
  Zo,
  Cache,
} from "@zero_one/client";
import Decimal from "decimal.js";
import { Zod } from "../../target/types/zod";
import { ZodBaseAccount } from "./ZodBaseAccount";
import ZodState from "./ZodState";
import { ZodMarginSchema } from "./zodTypes";

const ZOD_TOKEN_ACCOUNT_SIZE = 165;

interface Schema extends Omit<ZodMarginSchema, "collateral"> {
  /** The deposit amount divided by the entry supply or borrow multiplier */
  rawCollateral: Decimal[];
  /** The collateral value after applying supply/ borrow APY (i.e. the raw collateral multiplied by the current supply or borrow multiplier). */
  //actualCollateral: Num[];
}

export default class ZodMargin extends ZodBaseAccount<Schema> {
  private constructor(
    zodProgram: Program<Zod>,
    zoProgram: Program<Zo>,
    pubkey: PublicKey,
    data: Schema,
    public readonly zodState: ZodState,
    public readonly zodTokenAccount: PublicKey,
  ) {
    super(zodProgram, zoProgram, pubkey, data);
  }

  static async load(
    zodProgram: Program<Zod>,
    zoProgram: Program<Zo>,
    zodState: ZodState,
    zoState: State,
    cache: Cache,
    zodTokenAccount: PublicKey,
  ): Promise<ZodMargin> {
    const [zodMarginKey, zodMarginNonce] = await PublicKey.findProgramAddress(
      [
        zodProgram.provider.wallet.publicKey.toBuffer(),
        zodState.pubkey.toBuffer(),
        anchor.utils.bytes.utf8.encode("zodmarginv2"),
      ],
      zodProgram.programId,
    );

    console.log("zodMargin: ", zodMarginKey.toString())
    console.log("zodTokenAcc: ", zodTokenAccount.toString()) 

    const data = await this.fetch(zodProgram, zodMarginKey, zoState, cache);
    return new this(
      zodProgram,
      zoProgram,
      zodMarginKey,
      data,
      zodState,
      zodTokenAccount,
    );
  }

  static async create(
    zodProgram: Program<Zod>,
    zoProgram: Program<Zo>,
    zodState: ZodState,
    zoState: State,
    cache: Cache,
    commitment: Commitment = "finalized",
  ): Promise<ZodMargin> {
    const conn = zodProgram.provider.connection;

    const [zodMarginKey, zodMarginNonce] = await PublicKey.findProgramAddress(
      [
        zodProgram.provider.wallet.publicKey.toBuffer(),
        zodState.pubkey.toBuffer(),
        anchor.utils.bytes.utf8.encode("zodmarginv2"),
      ],
      zodProgram.programId,
    );

    const zodTokenAcc = await Keypair.generate();

    const zodTokenAccLamports = await conn.getMinimumBalanceForRentExemption(
      AccountLayout.span,
    );

    console.log({
      zodState: zodState.pubkey.toString(),
      payer: zodProgram.provider.wallet.publicKey.toString(),
      margin: zodMarginKey.toString(),
      tokenAccount: zodTokenAcc.publicKey.toString(),
      tokenProgram: TOKEN_PROGRAM_ID.toString(),
      mint: zodState.mint.toString(),
      authority: zodProgram.provider.wallet.publicKey.toString(),
      rent: SYSVAR_RENT_PUBKEY.toString(),
      systemProgram: SystemProgram.programId.toString(),
    })

    const tx = await zodProgram.rpc.createZodMargin(zodMarginNonce, {
      accounts: {
        zodState: zodState.pubkey,
        payer: zodProgram.provider.wallet.publicKey,
        margin: zodMarginKey,
        tokenAccount: zodTokenAcc.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        mint: zodState.mint,
        authority: zodProgram.provider.wallet.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      },
      preInstructions: [
        SystemProgram.createAccount({
          fromPubkey: zoProgram.provider.wallet.publicKey,
          newAccountPubkey: zodTokenAcc.publicKey,
          lamports: zodTokenAccLamports,
          space: AccountLayout.span,
          programId: TOKEN_PROGRAM_ID,
        }),
      ],
      signers: [zodTokenAcc],
    });

    console.log("tx:", tx);

    await conn.confirmTransaction(tx, commitment);

    return await ZodMargin.load(
      zodProgram,
      zoProgram,
      zodState,
      zoState,
      cache,
      zodTokenAcc.publicKey,
    );
  }

  private static async fetch(
    program: Program<Zod>,
    k: PublicKey,
    st: State,
    ch: Cache,
  ): Promise<Schema> {
    const data = (await program.account["zodMargin"].fetch(
      k,
    )) as ZodMarginSchema;
    //@ts-ignore
    const rawCollateral = data.collateral.map((c) => loadWI80F48(c!))
      .slice(0, st.data.totalCollaterals);
      return {
        ...data,
        rawCollateral,
        // actualCollateral: st.data.collaterals.map((c, i) => {
        //   return new Num(
        //     new BN(
        //       rawCollateral[i]!.isPos()
        //         ? rawCollateral[i]!.times(
        //             ch.data.borrowCache[i]!.supplyMultiplier,
        //           )
        //             .floor()
        //             .toString()
        //         : rawCollateral[i]!.times(
        //             ch.data.borrowCache[i]!.borrowMultiplier,
        //           )
        //             .floor()
        //             .toString(),
        //     ),
        //     c.decimals,
        //   );
        // }),
      };
  }

  async depositRaw(
    tokenAccount: PublicKey,
    zodVault: PublicKey,
    zoVault: PublicKey,
    stateSigner: PublicKey,
    cache: PublicKey,
    amount: BN,
  ) {
    return await this.zodProgram.rpc.zodDeposit(amount, {
      accounts: {
        zodState: this.zodState.pubkey,
        zoProgramMargin: this.zodState.data.zoProgramMargin,
        zoProgram: this.program.programId,
        zoProgramState: this.zodState.data.zoProgramState,
        stateSigner: stateSigner,
        cache: cache,
        authority: this.wallet.publicKey,
        zodMargin: this.pubkey,
        tokenAccount: tokenAccount,
        zoVault: zoVault,
        zodVault: zodVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
  }

  async withdrawRaw(
    tokenAccount: PublicKey,
    zodVault: PublicKey,
    zoVault: PublicKey,
    stateSigner: PublicKey,
    cache: PublicKey,
    control: PublicKey,
    amount: BN,
  ) {
    return await this.zodProgram.rpc.zodWithdraw(amount, {
      accounts: {
        zoProgramMargin: this.zodState.data.zoProgramMargin,
        zoProgram: this.program.programId,
        zodState: this.zodState.pubkey,
        zoProgramState: this.zodState.data.zoProgramState,
        stateSigner: stateSigner,
        cache: cache,
        authority: this.wallet.publicKey,
        zodMargin: this.pubkey,
        control: control,
        zodAccount: this.zodTokenAccount,
        tokenAccount: tokenAccount,
        zoVault: zoVault,
        zodVault: zodVault,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
  }

  async mintRaw(tokenAccount: PublicKey, cache: PublicKey, amount: BN) {
    return await this.zodProgram.rpc.zodMint(amount, {
      accounts: {
        zodState: this.zodState.pubkey,
        zoProgramState: this.zodState.data.zoProgramState,
        cache: cache,
        authority: this.wallet.publicKey,
        zodMargin: this.pubkey,
        tokenAccount: tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        mint: this.zodState.mint,
      },
    });
  }

  async burnRaw(tokenAccount: PublicKey, cache: PublicKey, amount: BN) {
    return await this.zodProgram.rpc.zodBurn(amount, {
      accounts: {
        zodState: this.zodState.pubkey,
        authority: this.wallet.publicKey,
        zodMargin: this.pubkey,
        tokenAccount: tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        mint: this.zodState.mint,
      },
    });
  }

  async liquidate(
    tokenAccount: PublicKey,
    liqee_margin: PublicKey,
    quoteMint: PublicKey,
    cache: PublicKey,
    amount: BN,
    testPrice?: BN,
  ) {
    return await this.zodProgram.rpc.liquidateZodPosition(amount, testPrice, {
      accounts: {
        zodState: this.zodState.pubkey,
        zoProgramState: this.zodState.data.zoProgramState,
        cache: cache,
        liqor: this.wallet.publicKey,
        liqorZodMargin: this.pubkey,
        liqeeZodMargin: liqee_margin,
        zodMint: this.zodState.mint,
        quoteMint: quoteMint,
        tokenAccount: tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
  }

  async settleBankruptcy(
    tokenAccount: PublicKey,
    liqee_margin: PublicKey,
    quoteMint: PublicKey,
    cache: PublicKey,
    testPrice?: BN,
  ) {
    return await this.zodProgram.rpc.zodSettleBankruptcy(testPrice, {
      accounts: {
        zodState: this.zodState.pubkey,
        zoProgramState: this.zodState.data.zoProgramState,
        cache: cache,
        liqor: this.wallet.publicKey,
        liqorZodMargin: this.pubkey,
        liqeeZodMargin: liqee_margin,
        zodMint: this.zodState.mint,
        quoteMint: quoteMint,
        tokenAccount: tokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });
  }

  //TODO
  async refresh(): Promise<void> {}
}
