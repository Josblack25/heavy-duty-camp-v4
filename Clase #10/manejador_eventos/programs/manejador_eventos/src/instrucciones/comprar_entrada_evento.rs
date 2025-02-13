use anchor_lang::prelude::*;
use anchor_spl::token::*;
use crate::colecciones::*;
use crate::utilidades::*;

/*
 Esta instruccion se invoca cuando un usuario "comprador" desea comprar una cantidad específica de entradas.
 Se debitan de la cuenta token de token aceptado del comprador el equivalente a la cantidad de entradas a comprar 
 * el precio de la entrada. Estos tokens aceptado son depositadon en la bóveda de ganancias del evento.
 
 - evento: Cuenta del evento que contienen toda la infomacion relacionada con el mismo.
 - cuenta_comprador_token_aceptado: cuenta de tokens aceptado propiedad del comprador de donde se debita el monto de las entradas compradas.
 - boveda_ganancias: cuenta del evento que almacena las ganancias de la venta de entradas del evento.
 - comprador: usuario que adquiere las entradas
 */

#[derive(Accounts)]
pub struct ComprarEntradaEvento<'info> {
    // CUENTAS
    #[account(
       mut, 
       seeds = [ 
            evento.id.as_ref(),
            Evento::SEMILLA_EVENTO.as_bytes(),
            evento.autoridad.as_ref(),
        ],
        bump = evento.bump_evento,
    )]
    pub evento: Account<'info, Evento>,

    // Cuenta de tokens aceptado del comprador
    #[account(
        mut,
        // chequeamos que contenga el token necesatio
        constraint = cuenta_comprador_token_aceptado.mint == evento.token_aceptado @ CodigoError::TokenIncorrecto,
        // chequeamos que tenga saldo
        constraint = cuenta_comprador_token_aceptado.amount > 0 @ CodigoError::SaldoInsuficiente
    )]
    pub cuenta_comprador_token_aceptado: Account<'info, TokenAccount>,

    // cuenta que almacenará los tokens aceptados provenientes
    // de esta venta de entradas del evento
    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_BOVEDA_GANANCIAS.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_boveda_ganancias,
    )]
    pub boveda_ganancias: Account<'info, TokenAccount>,

    // usuario que compra las entradas
    #[account(mut)]
    pub comprador: Signer<'info>, // usuario que compra los tokens

    //PROGRAMAS
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn comprar_entrada_evento(ctx: Context<ComprarEntradaEvento>, cantidad: u64) -> Result<()> {
    // cpi simple

    // total a cobrar = cantidad de entradas a comprar * el precio de la entrada
    let total = ctx.accounts.evento.precio_entrada * cantidad;

    let ctx_transferencia = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        // cuentas
        Transfer {
            from: ctx
                .accounts
                .cuenta_comprador_token_aceptado
                .to_account_info(), // de la cuenta de tokens aceptados del comprador
            to: ctx.accounts.boveda_ganancias.to_account_info(), // a la boveda de ganancias del evento
            authority: ctx.accounts.comprador.to_account_info(), // fimra el comprador
        },
    );

    // llamamos CPI para hacer la transferencia
    transfer(ctx_transferencia, total)?;

    // actualizamos losd atos del evento
    ctx.accounts.evento.entradas_vendidas += cantidad;

    Ok(())
}
