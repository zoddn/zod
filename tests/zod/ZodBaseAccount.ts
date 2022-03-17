import { Program } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { Wallet, Zo } from "@zero_one/client";
import { Zod } from "../../target/types/zod";

//had trouble importing this from zo
export default abstract class BaseAccount<T> {
  protected constructor(
    private _program: Program<Zo>,
    public readonly pubkey: PublicKey,
    public data: Readonly<T>,
  ) {}

  get program() {
    return this._program;
  }

  get provider() {
    return this.program.provider;
  }

  get connection() {
    return this.provider.connection;
  }

  get wallet(): Wallet {
    return this.provider.wallet;
  }

  abstract refresh(): Promise<void>;
}

/**
 * Base implementation of zod account classes
 */
export abstract class ZodBaseAccount<T> extends BaseAccount<T> {
  protected constructor(
    private _zodProgram: Program<Zod>,
    _program: Program<Zo>,
    readonly pubkey: PublicKey,
    data: Readonly<T>,
  ) {
    super(_program, pubkey, data);
  }

  get zodProgram() {
    return this._zodProgram;
  }
}
