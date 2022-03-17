import { PublicKey } from "@solana/web3.js";
import { BN } from "@project-serum/anchor";

export const ZO_DEX_PROGRAM_ID = new PublicKey(
  "CX8xiCu9uBrLX5v3DSeHX5SEvGT36PSExES2LmzVcyJd",
);
export const FUTURE_TAKER_FEE = 0.001; // 10bps
export const STARTING_FUNDING_INDEX = 10 ** 9;
export const STARTING_MULTIPLIER = 10 ** 6;
export const ONE_SIDED_MULTIPLIER = 0.05;
export const FUTURE_PERP_TYPE = {
  future: {},
};
export const EVER_C_PERP_TYPE = {
  callOption: {},
};
export const EVER_P_PERP_TYPE = {
  putOption: {},
};

export const BTC_DECIMALS = 6; // wBTC
export const BTC_OG_FEE = 10; // bps
export const BTC_LOT_SIZE = new BN(100);
export const USDC_BTC_LOT_SIZE = new BN(10);
export const BTC_BASE_IMF = new BN(100);
export const BTC_LIQ_FEE = new BN(20);
export const BTC_PERP_SYMBOL = "BTC-PERP";

export const SOL_DECIMALS = 9;
export const SOL_OG_FEE = 10; // bps
export const SOL_LOT_SIZE = new BN(100_000_000);
export const USDC_SOL_LOT_SIZE = new BN(100);
export const SOL_BASE_IMF = new BN(100);
export const SOL_LIQ_FEE = new BN(20);
export const SOL_EVER_C_SYMBOL = "SOL-EVER-C-200";
export const SOL_EVER_P_SYMBOL = "SOL-EVER-P-400";

export const USDC_DECIMALS = 6; // USD
export const USDC_OG_FEE = 10; // bps

export const USDC_ORACLE_SYM = "USDC/USD";
export const PYTH_USDC = new PublicKey(
  "5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7",
);
export const SWITCH_USDC = new PublicKey(
  "CZx29wKMUxaJDq6aLVQTdViPL754tTR64NAgQBUGxxHb",
);
export const BTC_ORACLE_SYM = "BTC/USD";
export const PYTH_BTC = new PublicKey(
  "HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J",
);
export const SWITCH_BTC = new PublicKey(
  "74YzQPGUT9VnjrBz8MuyDLKgKpbDqGot5xZJvTtMi6Ng",
);
export const PYTH_SOL = new PublicKey(
  "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix",
);
export const SWITCH_SOL = new PublicKey(
  "AdtRGGhmqvom3Jemp5YNrxd9q9unX36BZk1pujkkXijL",
);
export const MILLION = 10 ** 6;
