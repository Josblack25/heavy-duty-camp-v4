use anchor_lang::prelude::*;
use anchor_spl::associated_token::*;
use anchor_spl::token::*;
use crate::colecciones::*;
use crate::utilidades::*;

/*
 Esta instruccion se invoca cuando un usuario organizador del evento (quien lo crea y figura como autoridad) decide retirar fondos
 de la boveda del evento.

 - evento: Cuenta del evento que contienen toda la infomacion relacionada con el mismo.
 - token_aceptado: cuenta Mint de los tokens aceptados en las transacciones del evento.
 - cuenta_token_aceptado_autoridad: cuenta de tokens aceptado propiedad de la autoridad del evento donde se depositaran
 los tokens retirados de la boveda.
 - boveda_evento: cuenta del evento que almacena las ganancias de la venta de tokens del evento.
 - autoridad: usuario que retira los fondos.
 */

#[derive(Accounts)]
#[instruction(cantidad: u64)]
pub struct RetirarFondos<'info> {
    // CUENTAS
    #[account(
       mut, 
       seeds = [ 
            evento.id.as_ref(),
            Evento::SEMILLA_EVENTO.as_bytes(),
            autoridad.key().as_ref(),
        ],
        bump = evento.bump_evento,
        constraint = evento.autoridad == autoridad.key() @ CodigoError::UsuarioNoAutorizado, // el usuario esta autorizado
    )]
    pub evento: Account<'info, Evento>, // cuenta del evento

    // cuenta a donde se enviaran los tokens aceptados retirados de la boveda del evento
    #[account(
        init_if_needed,
        payer = autoridad, 
        associated_token::mint = token_aceptado, 
        associated_token::authority = autoridad, 
    )]
    pub cuenta_token_aceptado_autoridad: Account<'info, TokenAccount>,

    // boveda del evento de donde se debitaran los tokens retirados
    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_BOVEDA_EVENTO.as_ref(), 
            evento.key().as_ref()
        ],
        bump = evento.bump_boveda_evento,
        constraint = boveda_evento.amount >= (cantidad * 10u64.pow(token_aceptado.decimals.into())) @ CodigoError::SaldoInsuficiente,
      )]
    pub boveda_evento: Account<'info, TokenAccount>,

    // mint del token aceptado
    pub token_aceptado: Account<'info, Mint>,

    // la autoridad debe ser el usuraio organizador del evento
    #[account(mut)]
    pub autoridad: Signer<'info>, // usuario que creo el evento

    //PROGRAMAS
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn retirar_fondos(ctx: Context<RetirarFondos>, cantidad: u64) -> Result<()> {
    // La cuenta del Evento que es una PDA es la autoridad sobre la boveda del evento
    // por lo que necesitamos sus semillas para firmar
    let semillas_firma: &[&[&[u8]]] = &[&[
        ctx.accounts.evento.id.as_ref(),        // id del evento
        Evento::SEMILLA_EVENTO.as_bytes(),      // "evento"
        ctx.accounts.evento.autoridad.as_ref(), // pubKey de la autoridad
        &[ctx.accounts.evento.bump_evento],     // bump
    ]];

    // total a retirar = cantidad de tokens a retirar * 10 ^ decimales_token_aceptado
    let total = cantidad * 10_u64.pow(ctx.accounts.token_aceptado.decimals.into()); // expresado en la unidad minimal del token

    // contexto de la transfrerencia
    let ctx_transferencia = CpiContext::new(
        ctx.accounts.token_program.to_account_info(), // token program
        //cuentas
        Transfer {
            from: ctx.accounts.boveda_evento.to_account_info(), // de la boveda del evento
            to: ctx
                .accounts
                .cuenta_token_aceptado_autoridad
                .to_account_info(), // a la cuenta de token aceptado de la autoridad
            authority: ctx.accounts.evento.to_account_info(),   // firma el evento PDA
        },
    ).with_signer(semillas_firma); // firma el evento PDA

    // llamamos CPI para hacer la transferencia
    transfer(ctx_transferencia, total)?;
    Ok(())
}
