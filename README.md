# zod

To Run Zod I usually do `run anchor test ./tests/zod/zod-e2e.ts -- --features devnet` to test on devnet
then `cat .anchor/program-logs/<something here>.zod.log` to check logs

# This is how it works
<img width="1334" alt="Screen Shot 2022-03-17 at 12 35 24 AM" src="https://user-images.githubusercontent.com/101758471/158737919-d551bb8f-502b-481f-aae5-a08a123fd778.png">

# liquidation and insurance
The hard part of making this protocol was thinking of how the liquidation and insurance mechanics work. The program provides an instruction (in liquidate.rs) that people can use to liquidate other people if they are below the maintanence marginal fraction (MMF). I used similar logic to how 01 lending market liqquidation works when someone borrows usdc with other assets as collateral. The only difference here is that, instead of the liquidators paying back the zod, they will burn zod instead. This way the peg will be maintained. By decentralizing the liquidation process, anyone can profit by simply liquidating people who minted too much zod. As an incentive for helping maintain the peg of zod, they will be rewarded with a obtaining the collateral at a discount (liquidation fee). 

If a users collateral is already completely liquidated and there is still some outstanding zod minted balance by the user, then that user can still be liquidated. Since the user has no more collateral, liqquidators will be rewarded by fees directly from the insurance fund. If the insurance fund is finished, then the loss will be socialized and everyones zod minted balance will be increased instead.  
