use anchor_lang::prelude::*;
use anchor_spl::token::*;
use crate::colecciones::*;
use crate::utilidades::*;

/*
Esta instruccion va a consistir en eliminar la cuenta que almacena
la informacion del evento y las cuentas PDAS relacionadas al mismo
(boveda del evento, boveda de ganancias, token del evento)

Cuentas:
- evento: cuenta que almacena la información del evento
- boveda del evento: cuenta que almacena los tokens obtenidos de los colaboradores
- boveda de ganancias: cuentq que almacena los tokens obtenidos de la venta de entradas
- token_evento: token que se entrega a los colaboradores del evento 
- autoridad: usuario que creo el evento y que será quien puede eliminarlo
 */

// definimos el contexto
#[derive(Accounts)]
pub struct EliminarEvento<'info> {
    // CUENTAS
    #[account(
       mut, 
       seeds = [ 
            evento.id.as_ref(),
            Evento::SEMILLA_EVENTO.as_bytes(),
            autoridad.key().as_ref(),
        ],
        bump = evento.bump_evento,
        // verificaciones previas a elimimanr la cuenta
        constraint = evento.total_sponsors == 0 @ CodigoError::EventoConSponsors, // no hay sponsors todavía
        constraint = evento.autoridad == autoridad.key() @ CodigoError::UsuarioNoAutorizado, // el usuario esta autorizado
        close = autoridad // cierra automaticamente la cuenta evento y devuelve la renta al usuario autoridad
    )]
    pub evento: Account<'info, Evento>,

    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_BOVEDA_EVENTO.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_boveda_evento,
        // verificaciones previas a eliminar la cuenta
        constraint = boveda_evento.amount == 0 @ CodigoError::BovedaDelEventoNoVacia,  // debe estar vacía
    )]
    pub boveda_evento: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_BOVEDA_GANANCIAS.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_boveda_ganancias,
        //verificaciones previas a eliminar la cuenta
        constraint = boveda_ganancias.amount == 0 @ CodigoError::BovedaDeGananciasNoVacia, // debe estar vacía
    )]
    pub boveda_ganancias: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            Evento::SEMILLA_TOKEN_EVENTO.as_bytes(),
            evento.key().as_ref(),
        ],
        bump = evento.bump_token_evento,
    )]
    pub token_evento: Account<'info, Mint>, // vamos a "desactivar" esta cuenta

    #[account(mut)]
    pub autoridad: Signer<'info>, // usuario que creo el evento

    //PROGRAMAS
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/*
Esta función es válida para el caso en el que no hay 
ningun colaborador y no se han vendido tickets, e decir: 
boveda de ganancias en 0 tokens
boveda del evento en 0 tokens
 */
pub fn eliminar_evento(ctx: Context<EliminarEvento>) -> Result<()> {
    // La cuenta del Evento que es una PDA es la autoridad sobre todas las cuentas a eliminar
    // por lo que necesitamos sus semillas para firmar
    let semillas_firma: &[&[&[u8]]] = &[&[
        ctx.accounts.evento.id.as_ref(),        // id del evento
        Evento::SEMILLA_EVENTO.as_bytes(),      // "evento"
        ctx.accounts.evento.autoridad.as_ref(), // pubKey de la autoridad
        &[ctx.accounts.evento.bump_evento],     // bump
    ]];

    /*
    Cerramos las cunetas de las bóvedas:
    Para esto hacemos un CPI a la instrucción CloseAccount
    del token program.
     */

    // cerramos la boveda del evento
    let cerrar_boveda_evento = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.boveda_evento.to_account_info(), // cuenta a cerrar
            destination: ctx.accounts.autoridad.to_account_info(), // se devuelve la renta al usuario incializador
            authority: ctx.accounts.evento.to_account_info(),      // el evento debe autorizar
        },
    ).with_signer(semillas_firma); // firma con PDA

    // llamamos a la CPI
    close_account(cerrar_boveda_evento)?;

    // cerramos la boveda de ganacias
    let cerrar_boveda_ganancias = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.boveda_ganancias.to_account_info(), // cuenta a cerrar
            destination: ctx.accounts.autoridad.to_account_info(), // se devuelve la renta al usuario incializador
            authority: ctx.accounts.evento.to_account_info(),      // el evento debe autorizar
        },
    ).with_signer(semillas_firma); // firma con PDA

    //  llamamos a la CPI
    close_account(cerrar_boveda_ganancias)?;

    /*
    Una vez eliminadas las cuentas de las bóvedas, lo que sigue es
    eliminar la cuenta Mint del token del evento, sin embargo, esto
    solo se puede hacer si se utiliza el token program con extensiones.
    Al haber utilizado el token program tradicional, podemos "desactivar" 
    la cuenta mint, revocando los permisos de autoridad de mint
    */

    // Como DESACTIVAR una cuenta mint:
    // el supply DEBE ser 0 -> esto se cuemple ya que no hay colaboradores
    // no puede tener mint authority

    // revocamos la autoridad
    let revocar_autoridad = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SetAuthority {
            account_or_mint: ctx.accounts.token_evento.to_account_info(), // cuenta Mint a actualizar
            current_authority: ctx.accounts.evento.to_account_info(),     // autoridad a revocar
        },
    ).with_signer(semillas_firma); // el evento firma PDA

    // llamamos a la CPI
    set_authority(
        revocar_autoridad,
        spl_token::instruction::AuthorityType::MintTokens, // tipo de autoridad a revocar, en este caso mint authority
        None, // nueva autoridad -> para "desactivar" debe ser None
    )?;

    // con el Token extensions podriamos cerrar la cuenta Mint al cerrar el evento

    Ok(())
}
