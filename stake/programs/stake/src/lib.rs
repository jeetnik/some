use anchor_lang::prelude::*;
use anchor_lang::system_program;


declare_id!("2KxbEFvQZ3ojqHLTAe9XN59wpuuD4nrFVwBkvgwNELY2");

#[program]
pub mod stake {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let staking_pool = &mut ctx.accounts.staking_pool;
        staking_pool.authority = ctx.accounts.authority.key();
        staking_pool.total_staked = 0;
        staking_pool.reward_rate = 1; // 1 lamport per second per SOL staked
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // Transfer SOL from user to staking pool
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.staking_pool.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        // Update staking pool and user stake
        let staking_pool = &mut ctx.accounts.staking_pool;
        let user_stake = &mut ctx.accounts.user_stake;

        user_stake.user = ctx.accounts.user.key();
        user_stake.amount = user_stake.amount.checked_add(amount).unwrap();
        user_stake.last_stake_timestamp = Clock::get()?.unix_timestamp;
        staking_pool.total_staked = staking_pool.total_staked.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let staking_pool = &mut ctx.accounts.staking_pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // Check if user has enough staked amount
        if amount > user_stake.amount {
            return err!(ErrorCode::InsufficientStake);
        }

        // Transfer SOL back to user
        let bump = ctx.bumps.staking_pool;
        let seeds = &[b"staking_pool", &[bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.staking_pool.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        );
        system_program::transfer(cpi_context, amount)?;

        // Update staking pool and user stake
        user_stake.amount = user_stake.amount.checked_sub(amount).unwrap();
        staking_pool.total_staked = staking_pool.total_staked.checked_sub(amount).unwrap();

        // Update last stake timestamp
        user_stake.last_stake_timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let staking_pool = &mut ctx.accounts.staking_pool;
        let user_stake = &mut ctx.accounts.user_stake;

        // Calculate reward
        let current_time = Clock::get()?.unix_timestamp;
        let time_staked = current_time - user_stake.last_stake_timestamp;
        let reward = user_stake.amount
            .checked_mul(staking_pool.reward_rate)
            .unwrap()
            .checked_mul(time_staked as u64)
            .unwrap();

        // Transfer reward to user
        let bump = ctx.bumps.staking_pool;
        let seeds = &[b"staking_pool", &[bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.staking_pool.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            signer_seeds,
        );
        system_program::transfer(cpi_context, reward)?;

        // Update last stake timestamp
        user_stake.last_stake_timestamp = current_time;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info>{
    #[account(init,
        payer=authority,
        bump ,
        seeds=[b"staking_pool"],
        space = 8 + 32 + 8 + 8 
    )]
    pub staking_pool:Accounts<'info,StakingPool>,
    #[account(mut)]
    pub authority:Signer<'info>,
    pub system_program:Program<'info,System>
}

#[derive(account)]
#[instruction(amount: u64)]
pub struct Stake<'info>{
    #[account(
        mut,
        seeds = [b"staking_pool"],
        bump
    )]//initialize mein humne seeds = [b"staking_pool"] use karke ek global PDA banaya tha.
    //Ab stake, unstake, ya claim_reward mein hum usi PDA ko use kar rahe hain.
    pub staking_pool: Account<'info, StakingPool>,
    #[account(
        init_if_needed,
        seeds = [b"user_stake", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 32 + 8 + 8 // discriminator + user + amount + last_stake_timestamp
    )]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

}

#[derive(account)]
#[instruction(amount:u64)]
pub struct Unstake<'info>{
    #[account(mut,seeds=[b"staking_pool"],bump)]
    pub staking_pool:Account<'info,StakingPool>
    #[account(mut,seeds=[b"user_stake",user.key().as_ref()])]
    pub user_stake:Account<'info ,UserStake>
    #[account(mut)]
    pub user:Signer<'info>,
    pub system_program:Program<'info,Program>
}
#[derive(account)]
pub struct ClaimReward<'info>{
    #[account(mut,seeds=[b"staking_pool"],bump)]
    pub staking_pool:Account<'info, StakingPool>,
    #[account(
        mut,
        seeds = [b"user_stake", user.key().as_ref()],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub total_staked: u64,
    pub reward_rate: u64,
}

#[account]
pub struct UserStake {
    pub user: Pubkey,
    pub amount: u64,
    pub last_stake_timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient staked amount")]
    InsufficientStake,
}