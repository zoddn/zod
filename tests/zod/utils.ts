import * as anchor from "@project-serum/anchor";
import { Provider, web3 } from "@project-serum/anchor";
import { TokenInstructions } from "@project-serum/serum";
import { PublicKey } from "@solana/web3.js";
import { getTokenAccount } from "../../deps/zo-client";
import { u64 } from "@solana/spl-token";

export async function withBalanceChange(
  provider: Provider,
  addrs: PublicKey[],
  fn,
) {
  const beforeBalances: u64[] = [];

  // console.log(await getTokenAccount(provider, addrs[1]!));

  // console.log("here!!!!");
  for (let k = 0; k < addrs.length; k += 1) {
    beforeBalances.push((await getTokenAccount(provider, addrs[k]!)).amount);
  }

  await fn();

  const afterBalances: u64[] = [];
  for (let k = 0; k < addrs.length; k += 1) {
    afterBalances.push((await getTokenAccount(provider, addrs[k]!)).amount);
  }

  const deltas: number[] = [];
  for (let k = 0; k < addrs.length; k += 1) {
    deltas.push(afterBalances[k]!.toNumber() - beforeBalances[k]!.toNumber());
  }
  return deltas;
}

export function approxEq(a: number, b: number, e?: number) {
  if (e == null) {
    e = 0.000_000_1;
  }
  if (Math.abs(a - b) < e) {
    return true;
  } else {
    console.log("a: ", a);
    console.log("b: ", b);
    return false;
  }
}

export function getRandomArbitrary(min, max) {
  return Math.random() * (max - min) + min;
}

export async function mintToAccount(
  provider: anchor.Provider,
  mint: anchor.web3.PublicKey,
  destination: anchor.web3.PublicKey,
  amount: anchor.BN,
  mintAuthority: anchor.web3.PublicKey,
): Promise<void> {
  const tx: anchor.web3.Transaction = new anchor.web3.Transaction();
  tx.add(
    ...(await createMintToAccountInstrs(
      mint,
      destination,
      amount,
      mintAuthority,
    )),
  );
  await provider.send(tx, []);
  return;
}

export async function createMintToAccountInstrs(
  mint: anchor.web3.PublicKey,
  destination: anchor.web3.PublicKey,
  amount: anchor.BN,
  mintAuthority: anchor.web3.PublicKey,
): Promise<anchor.web3.TransactionInstruction[]> {
  return [
    TokenInstructions.mintTo({
      mint,
      destination: destination,
      amount: amount,
      mintAuthority: mintAuthority,
    }),
  ];
}

export async function generateClientProgram<T>(
  // @ts-ignore
  program: anchor.Program<T>,
  // @ts-ignore
): Promise<anchor.Program<T>> {
  const acc = await web3.Keypair.generate();
  const provider = new anchor.Provider(
    program.provider.connection,
    // @ts-ignore
    new anchor.Wallet(acc),
    {
      preflightCommitment: "confirmed",
      commitment: "confirmed",
    },
  );
  //cant get airdrops working
  //await fundAccount(provider.connection, acc, web3.LAMPORTS_PER_SOL);
  // @ts-ignore
  return new anchor.Program(program.idl, program.programId, provider);
}

export async function fundAccount(
  conn: web3.Connection,
  account: web3.Keypair,
  lamports: number,
) {
  const sig = await conn.requestAirdrop(account.publicKey, lamports);
  await conn.confirmTransaction(sig, "confirmed");
}

export async function newAccountWithLamports(
  connection: anchor.web3.Connection,
  lamports = 1e10,
): Promise<anchor.web3.Keypair> {
  const account = new anchor.web3.Keypair();

  let retries = 30;
  await connection.requestAirdrop(account.publicKey, lamports);
  for (;;) {
    await sleep(500);
    // eslint-disable-next-line eqeqeq
    if (lamports == (await connection.getBalance(account.publicKey))) {
      return account;
    }
    if (--retries <= 0) {
      break;
    }
  }
  throw new Error(`Airdrop of ${lamports} failed`);
}

async function sleep(ms: number) {
  // @ts-ignore
  return new Promise((resolve) => setTimeout(resolve, ms));
}
