use anchor_lang::prelude::*;
use crate::colecciones::*;
use crate::utilidades::*;

// definimos el contexto
#[derive(Accounts)]
pub struct FinalizarEvento<'info> {
    #[account(
       mut, 
       seeds = [ 
            evento.id.as_ref(),
            Evento::SEMILLA_EVENTO.as_bytes(),
            autoridad.key().as_ref(),
        ],
        bump = evento.bump_evento,
        // verificaciones previas
        constraint = evento.autoridad == autoridad.key() @ CodigoError::UsuarioNoAutorizado, // el usuario esta autorizado
    )]
    pub evento: Account<'info, Evento>,

    #[account(mut)]
    pub autoridad: Signer<'info>, // solo el usuario que creo el evento puede finalizarlo

    //PROGRAMAS
    pub system_program: Program<'info, System>,
}

pub fn finalizar_evento(ctx: Context<FinalizarEvento>) -> Result<()> {
    ctx.accounts.evento.activo = false;
    Ok(())
}
