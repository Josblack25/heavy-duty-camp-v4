use anchor_lang::prelude::*;
use anchor_spl::associated_token::*;
use anchor_spl::token::*;
//programa
declare_id!("HvQZWizSS6sPkSw8j3KsTMuLjgpm6hPjiUKZ9F1bpVB7");

#[program]
pub mod tokens {

    use super::*;
    pub fn create_token_mint(_ctx: Context<CreateToken>) -> Result<()> {
        Ok(())
    }
}

// 1. crear el contexto
#[derive(Accounts)]
pub struct CreateToken<'info> {
    //2. crear cuenta mint
    #[account(init, payer=authority, mint::decimals = 6, mint::authority = authority)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = authority, 
        associated_token::mint = mint, 
        associated_token::authority = authority
    )]
    pub token_account: Account<'info, TokenAccount>,

    //autoridad del mint
    #[account(mut)]
    pub authority: Signer<'info>,

    //programas
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    //variables
    pub rent: Sysvar<'info, Rent>,
}