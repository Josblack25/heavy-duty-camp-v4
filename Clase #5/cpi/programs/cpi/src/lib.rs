use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("9AvUNHjxscdkiKQ8tUn12QCMXtcnbR9BVGq3ULNzFMRi");

#[program]
pub mod cpi {
    use super::*;

    pub fn transferir(ctx: Context<Transferir>, cantidad: u64) -> Result<()> {
        let de = ctx.accounts.remitente.to_account_info();
        let para = ctx.accounts.recipiente.to_account_info();
        let programa = ctx.accounts.system_program.to_account_info();

        let cpi_context = CpiContext::new(
            programa,
            Transfer {
                from: de,
                to: para,
            },
        );

        transfer(cpi_context, cantidad)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Transferir<'info> {
    #[account(mut)] // mutable porque se le debita saldo
    remitente: Signer<'info>,
    #[account(mut)] // mutable porque se le aumenta saldo
    recipiente: SystemAccount<'info>,
    system_program: Program<'info, System>,
}
