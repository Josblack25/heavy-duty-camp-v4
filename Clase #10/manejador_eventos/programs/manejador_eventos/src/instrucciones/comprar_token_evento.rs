use anchor_lang::prelude::*;
use anchor_spl::associated_token::*;
use anchor_spl::token::*;
use crate::colecciones::*;
use crate::utilidades::*;
/*
 Esta instruccion se invoca cuando un usuario "comprador" desea colaborar con el evento mediante la compra de tokens del evento
 Se debitan de la cuenta token de token aceptado del comprador el equivalente a la cantidad de tokens a comprar * el precio del 
 token del evento. Estos tokens aceptado son depositadon en la bóveda del evento.
 Una vez cobrados los tokens del evento, el programa hace Mint de la cantidad correspondiente de tokens del evento a la cuenta
 token del usuario comprador.

 - evento: Cuenta del evento que contienen toda la infomacion relacionada con el mismo.
 - cuenta_comprador_token_evento: cuenta de tokens del evento del comprador del token, si no existe anteriormente, se crea.
 - token_evento: cuenta Mint de los tokens del evento que adquiere el comprador.
 - cuenta_comprador_token_aceptado: cuenta de tokens aceptado propiedad del comprador de donde se debita el monto de los tokens comprados.
 - boveda_evento: cuenta del evento que almacena las ganancias de la venta de tokens del evento.
 - comprador: usuario que adquiere los tokens del evento
 */

#[derive(Accounts)]
pub struct ComprarTokenEvento<'info> {
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

    // cuenta de tokens del evento del comprador del token
    #[account(
        init_if_needed, // si la cuenta no existe el programa la crea
        payer = comprador, // el comprador coore con los gasto de renta de su cuenta
        associated_token::mint = token_evento, // la cuenta contiene tokens del evento
        associated_token::authority = comprador, // la autoridad de la cuenta es el comprador
    )]
    pub cuenta_comprador_token_evento: Account<'info, TokenAccount>,

    // Cuenta mint del token del evento (PDA)
    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_TOKEN_EVENTO.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_token_evento,
    )]
    pub token_evento: Account<'info, Mint>, // mint de este token al comprador

    // Cuenta de tokens aceptado del comprador
    #[account(
        mut,
        // chequeamos que contenga el token necesatio
        constraint = cuenta_comprador_token_aceptado.mint == evento.token_aceptado @ CodigoError::TokenIncorrecto,
        // chequeamos que tenga saldo
        constraint = cuenta_comprador_token_aceptado.amount > 0 @ CodigoError::SaldoInsuficiente

    )]
    pub cuenta_comprador_token_aceptado: Account<'info, TokenAccount>, // se debitaran el monto del precio de los tokens

    // cuenta que almacenará los tokens aceptados provenientes
    // de esta venta de tokens
    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_BOVEDA_EVENTO.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_boveda_evento,
    )]
    pub boveda_evento: Account<'info, TokenAccount>,

    // usuario que compra los tokens del evento (colaborador /sponsor)
    #[account(mut)]
    pub comprador: Signer<'info>, // usuario que compra los tokens

    //PROGRAMAS
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

// Funcion de la instruccion

pub fn comprar_token_evento(ctx: Context<ComprarTokenEvento>, cantidad: u64) -> Result<()> {
    // La cuenta del Evento que es una PDA es la autoridad sobre todas las cuentas a eliminar
    // por lo que necesitamos sus semillas para firmar
    let semillas_firma: &[&[&[u8]]] = &[&[
        ctx.accounts.evento.id.as_ref(),        // id del evento
        Evento::SEMILLA_EVENTO.as_bytes(),      // "evento"
        ctx.accounts.evento.autoridad.as_ref(), // pubKey de la autoridad
        &[ctx.accounts.evento.bump_evento],     // bump
    ]];

    // instruccion de 2 partes:
    // 1. transferir del colaborador a la cuenta del evento
    // 2. mintear los tokens del evento a la cuenta del colaborador

    // Transferimos
    // total a cobrar = cantidad de tokens a comprar * el precio del token
    let total = ctx.accounts.evento.precio_token * cantidad;

    // contexto para la transferencia de tokens aceptado desde la cuenta del comprador
    // hasta la cuenta boveda del evento
    let ctx_transferencia = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        //cuentas
        Transfer {
            from: ctx
                .accounts
                .cuenta_comprador_token_aceptado
                .to_account_info(), // de la cuenta token del comprador
            to: ctx.accounts.boveda_evento.to_account_info(), // a la boveda del evento
            authority: ctx.accounts.comprador.to_account_info(), // firma el comprador
        },
    );

    // llamamos CPI para hacer la transferencia
    transfer(ctx_transferencia, total)?;

    // Hacemos mint (2da parte de la isntruccion)

    // Una vez cobrados los tokens, hacemos mint de a cantidad comprada a la cuenta
    // de tokens del evento del comprador
    let ctx_mint = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        // cuentas
        MintTo {
            mint: ctx.accounts.token_evento.to_account_info(), // cuenta mint del token del evento
            to: ctx.accounts.cuenta_comprador_token_evento.to_account_info(), // minteamos a la cuenta del comrpador
            authority: ctx.accounts.evento.to_account_info(), // firma el evento (PDA)
        },
    ).with_signer(semillas_firma);

    // llamamos a la CPI para hacer mint
    mint_to(ctx_mint, cantidad)?;

    // nuestro evento guarda informacion de las compras dee tokens y entradas
    // actualizamos los datos del evento con respecto alos sponsors
    ctx.accounts.evento.tokens_vendidos += cantidad; // cantodad de tokens vendidos
    ctx.accounts.evento.total_sponsors += 1; // cantidad total de sponsors del evento
    ctx.accounts.evento.sponsors_actuales += 1; // cantidad actual de sponsors del evento

    Ok(())
}
