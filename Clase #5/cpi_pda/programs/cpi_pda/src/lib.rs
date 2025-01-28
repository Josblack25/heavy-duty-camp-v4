use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("3455LkCS85a4aYmSeNbRrJsduNQfYRY82A7eCD3yQfyR");

#[program]
pub mod cpi_pda {
    use super::*;

    pub fn transferencia(ctx: Context<Transferencia>, cantidad: u64) -> Result<()> {
        let de = ctx.accounts.pda_remitente.to_account_info();
        let para = ctx.accounts.recipiente.to_account_info();
        let programa = ctx.accounts.system_program.to_account_info();

        let bump_seed = ctx.bumps.pda_remitente;
        let signer_seeds: &[&[&[u8]]] = &[&[b"pda", &[bump_seed]]];

        let cpi_context = CpiContext::new(
            programa,
            Transfer {
                from: de,
                to: para,
            },
        )
        .with_signer(signer_seeds);

        transfer(cpi_context, cantidad)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Transferencia<'info> {
    #[account(
        mut,
        seeds = [b"pda"], 
        bump, 
    )]
    pda_remitente: SystemAccount<'info>,
    #[account(mut)]
    recipiente: SystemAccount<'info>,
    system_program: Program<'info, System>,
}

