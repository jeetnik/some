use anchor_lang::prelude::*;

declare_id!("Cntr1234u9zS8dEg5jMQKjzAqz9HmeXE8JDh8vAmsRf2");
#[program]
pub mod counter{
    use super::*;
 
   pub fn create(ctx:Context<Initialize>)->Result<()>{
    let counter=&mut ctx.accounts.counter;
    counter.data=0;
    Ok(())
   }

   pub fn increase(ctx:Context<Increase>)->Result<()>{
    let counter = &mut ctx.accounts.counter;
    counter.data += 1;
    Ok(())
   }

}

#[derive(Accounts)]

pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [b"counter", user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + 4 
    )]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct Increase<'info> {
    #[account(
        mut,
        seeds = [b"counter", user.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,
    pub user: Signer<'info>,
}

#[account]
pub struct Counter {
    pub data: u32,
}