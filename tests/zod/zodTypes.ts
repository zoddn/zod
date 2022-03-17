import { BN, IdlAccounts, IdlTypes } from "@project-serum/anchor";
import { Zo } from "../../deps/zo-client";
import { Zod } from "../../target/types/zod";

type Symbol = { data: number[] };
type WrappedI80F48 = { data: BN };

type ZodCollateralInfo = Omit<IdlTypes<Zod>["ZodCollateralInfo"], "oracleSymbol"> & {
  oracleSymbol: Symbol;
};

//just put here for cnvention
export type ZodStateSchema= Omit<IdlAccounts<Zod>["zodState"], "zodTokenInfo" | "totalZodBorrowed" | "socLossMultiplier"> & {
  zodTokenInfo: ZodCollateralInfo;
  totalZodBorrowed: WrappedI80F48;
  socLossMultiplier: WrappedI80F48;
}; 
export type ZodMarginSchema= Omit<IdlAccounts<Zod>["zodMargin"], "zodBalance"> & {
  zodBalance: WrappedI80F48;
};