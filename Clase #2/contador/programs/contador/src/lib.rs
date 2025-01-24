use anchor_lang::prelude::*;

//instruccion que creea un contador en Solana
// contador: número - inicia en 0

declare_id!("CWu4uabYiWxVArFUvtekBfBM7iGMResjFAizcKpQuCUm");

// 4. definimos el programa
#[program]
pub mod contador {

    use super::*;
    // 3. función de la instrucción
    pub fn crear_contador(ctx: Context<CrearContador>) -> Result<()> {
        // 5. codigo de la instrucción
        ctx.accounts.contador.valor = 0;
        Ok(()) // retorna Ok
    }

}

// 2. contexto de la instruccion
#[derive(Accounts)]
pub struct CrearContador<'info> {
    //3. cuentas
    #[account(init, payer = user, space = 8 + Contador::INIT_SPACE)]
    pub contador: Account<'info, Contador>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// 1. estructura de datos del contador (cuenta)
#[account]
#[derive(InitSpace)]
pub struct Contador {
    pub valor: u64, // entero sin signo de 64 bits - 8 bytes
}