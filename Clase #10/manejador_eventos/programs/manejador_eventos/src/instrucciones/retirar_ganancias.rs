use anchor_lang::prelude::*;
use anchor_spl::token::*;
use crate::colecciones::*;
use crate::utilidades::*;

/*
 Esta instruccion se invoca cuando un usuario colaborador (que adquirio tokens del evento anteriormente) decide
 retirar sus ganancias correspondientes de la boveda de ganancias del evento.
 El evento debe haber finalizado antes de que cualquier colaborador retire sus ganancias
 Esta instruccion se realiza en dos partes, primero se "queman" los tokens del evento que posee el usuario
 colaborador (para evitar que los cambie de nuevo por ganancias)
 Luego se retira de la boveda de ganancia el monto correspondiete al porcentaje de tokens del evento que 
 el usuario colaborador adquirio con respecto al total de tokens del evento vendidos.

 - evento: Cuenta del evento que contienen toda la infomacion relacionada con el mismo.
 - token_evento: cuenta Mint de los tokens del evento.
 - cuenta_colaborador_token_evento: cuneta de tokesn del evento propiedad del colaborador que almacena los tokens
 a quemar
 - cuenta_colaborador_token_aceptado: cuenta de tokens aceptado propiedad del usuario colaborador donde se depositaran
 sus ganancias correspondientes.
 - boveda_ganancias: cuenta del evento que almacena las ganancias de la venta de entradas del evento.
 - colaborador: usuario colaborador que retira sus ganancias.
 */

#[derive(Accounts)]
pub struct RetirarGanancias<'info> {
    // CUENTAS
    #[account(
       mut, 
       seeds = [ 
            evento.id.as_ref(),
            Evento::SEMILLA_EVENTO.as_bytes(),
            evento.autoridad.as_ref(),
        ],
        bump = evento.bump_evento,
        constraint = evento.activo == false @ CodigoError::EventoActivo
    )]
    pub evento: Account<'info, Evento>, // cuenta del evento

    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_TOKEN_EVENTO.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_token_evento,
    )]
    pub token_evento: Account<'info, Mint>, // vamos a "quemar" unidades de este token

    // Cuenta de tokens aceptado del colaborador donde depositaremos las ganancias
    #[account(
        mut,
        // chequeamos que contenga el token necesatio
        constraint = cuenta_colaborador_token_aceptado.mint == evento.token_aceptado @ CodigoError::TokenIncorrecto,
    )]
    pub cuenta_colaborador_token_aceptado: Account<'info, TokenAccount>,

    // Cuenta de tokens del evento del colaborador de donde se quemaran los tokens correspondientes
    #[account(
        mut,
        // chequeamos que contenga el token necesatio
        constraint = cuenta_colaborador_token_evento.mint == token_evento.key() @ CodigoError::TokenIncorrecto,
    )]
    pub cuenta_colaborador_token_evento: Account<'info, TokenAccount>,

    // boveda de ganacias que almacena los tokens obtenidos de la venta de entradas y de donde se debitaran
    // las ganancias del usuario colaborador
    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_BOVEDA_GANANCIAS.as_ref(), 
            evento.key().as_ref()
        ],
        bump = evento.bump_boveda_ganancias,
        
      )]
    pub boveda_ganancias: Account<'info, TokenAccount>,

    #[account(mut)]
    pub colaborador: Signer<'info>, // usuario colaborador que retira sus ganancias

    //PROGRAMAS
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn retirar_ganancias(ctx: Context<RetirarGanancias>) -> Result<()> {
    // calculamos los totales de los tokens del colaborador y de las bovedas del evento
    let total_token_vendidos = ctx.accounts.evento.tokens_vendidos; // total de tokens vendido (100% de los tokens de colaborador)
    let tokens_colaborador = ctx.accounts.cuenta_colaborador_token_evento.amount; // cantidad de tokens de colaborador que posee el usuario
    let total_boveda_ganancias = ctx.accounts.boveda_ganancias.amount; // total almacenado en la boveda de ganancias

    // calculamos la ganancia del colaborador

    // calculamos el porcentaje % de tokens del evento que el usuario colaborador posee con respecto
    // al total de tokens vendidos
    let porcentaje = calcular_porcentaje(total_token_vendidos, tokens_colaborador);

    // calculamos el total a transferir con el porcentaje anterior
    // con el % anterior calculamos el total de ganacias que le corresponden al colaborador segun su porcentaje
    let total_ganancias_colaborador = calcular_ganancias(total_boveda_ganancias, porcentaje);

    // instruccion de 2 partes
    // 1. Quemamos los tokens de colaborador
    // definimos el contexto
    let ctx_quemar = CpiContext::new(
        ctx.accounts.token_program.to_account_info(), // token program
        // cuentas
        Burn {
            mint: ctx.accounts.token_evento.to_account_info(), // mint de los tokes a quemar (tokens del evento)
            from: ctx
                .accounts
                .cuenta_colaborador_token_evento
                .to_account_info(), // cunata que almacena los tokens a quemar (cuenta de tokens del evento del colaborador)
            authority: ctx.accounts.colaborador.to_account_info(), // autoridad de lo tokens a quemar (usuario colaborador)
        },
    ); // cpi simple ya que firma el colaborador

    // llamamos a la instruccion quemar
    burn(ctx_quemar, tokens_colaborador)?; // quemamo el total de tokens del evento que tiene el colaborador

    //2.  Transferimos las ganancias a la cuenta de token aceptado del colaborador

    // La cuenta del Evento que es una PDA es la autoridad sobre la bobeva de ganancias
    // por lo que necesitamos sus semillas para firmar
    let semillas_firma: &[&[&[u8]]] = &[&[
        ctx.accounts.evento.id.as_ref(),        // id del evento
        Evento::SEMILLA_EVENTO.as_bytes(),      // "evento"
        ctx.accounts.evento.autoridad.as_ref(), // pubKey de la autoridad
        &[ctx.accounts.evento.bump_evento],     // bump
    ]];

    // definimos el contexto de la transferencia
    let ctx_transferencia = CpiContext::new(
        ctx.accounts.token_program.to_account_info(), //token porgram
        //cuentas
        Transfer {
            from: ctx.accounts.boveda_ganancias.to_account_info(), // de la boveda de genancias del evento
            to: ctx
                .accounts
                .cuenta_colaborador_token_aceptado
                .to_account_info(), // a la cuneta de tokens aceptado del colaborador
            authority: ctx.accounts.evento.to_account_info(),      // firma el evento (PDA)
        },
    ).with_signer(semillas_firma); // firma el evento (PDA)

    // llamamos CPI para hacer la transferencia
    transfer(ctx_transferencia, total_ganancias_colaborador)?;
    Ok(())
}
