use anchor_lang::prelude::*;
use anchor_spl::token::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("88fk1UW3DouQ8RpZa5sYME4k8dFH7yPt5nmkxpJXqz8G");

#[program]
mod hello_anchor {
    use super::*;
    pub fn inicializar(
        ctx: Context<Inicializar>,
        id: String,
        cantidad_tokens_a: u64,
        cantidad_tokens_b: u64,
    ) -> Result<()> {
        // almacenamos la infromacion del intercambio en la cuenta escrow
        ctx.accounts.escrow.inicializador = ctx.accounts.inicializador.key();
        ctx.accounts.escrow.token_a = ctx.accounts.token_a.key();
        ctx.accounts.escrow.token_b = ctx.accounts.token_b.key();
        ctx.accounts.escrow.id = id;

        // almacenamos las cantidades del intercambio para cada token (en decimales)
        // 100 * 10´0 = 100
        ctx.accounts.escrow.cantidad_token_a =
            cantidad_tokens_a * 10_u64.pow(ctx.accounts.token_a.decimals.into());

        // 95 * 10´0 = 95
        ctx.accounts.escrow.cantidad_token_b =
            cantidad_tokens_b * 10_u64.pow(ctx.accounts.token_b.decimals.into());

        // almacenamos los bumps
        ctx.accounts.escrow.bump_escrow = ctx.bumps.escrow;
        ctx.accounts.escrow.bump_cuenta_garantia = ctx.bumps.cuenta_de_garantia;

        /* 
        Transferimos la cantidad de token_A que el incializador está dispuesto
        a intercambiar desde la cuenta token del inicializador a la cuenta
        de garantía, donde permanecerán hasta que se cancele o finalice el 
        intercambio de tokens
        */
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.inicializador_cuenta_token_a.to_account_info(),
                to: ctx.accounts.cuenta_de_garantia.to_account_info(),
                authority: ctx.accounts.inicializador.to_account_info(),
            },
        );

        // llamamos a la cpi
        transfer(cpi_ctx, ctx.accounts.escrow.cantidad_token_a)?;

        Ok(())
    }

    pub fn finalizar(ctx: Context<Finalizar>) -> Result<()> {
        /* 
        Transferimos los token_B de la cuenta token del aceptante 
        a la cuenta token del inicializador
        */

        let cpi_al_inicializador = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.aceptante_token_account_b.to_account_info(),
                to: ctx.accounts.inicializador_token_account_b.to_account_info(),
                // el que acepta la oferta tiene que firmar desde el client
                authority: ctx.accounts.aceptante.to_account_info(),
            },
        );

        // llamada a la cpi
        transfer(cpi_al_inicializador, ctx.accounts.escrow.cantidad_token_b)?;

        /*
       Una vez que el usuario incializador recibe los token_B del intercambio, el programa
       tranfiere de la cuenta de garantia (PDA) los token_A hacia la cuenta token
       del usuario aceptante.
        */

        // definimos las semillas firmantes para la PDA
        let semillas_firma: &[&[&[u8]]] = &[&[
            ctx.accounts.escrow.to_account_info().key.as_ref(),
            &[ctx.accounts.escrow.bump_cuenta_garantia],
        ]];

        // transferimos los tokens de la cuenta de garantía a la cuenta token del usuario aceptante
        let cpi_al_aceptante = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.cuenta_de_garantia.to_account_info(),
                to: ctx.accounts.aceptante_token_account_a.to_account_info(),
                authority: ctx.accounts.cuenta_de_garantia.to_account_info(),
            },
        ).with_signer(semillas_firma);

        // llamada cpi
        transfer(cpi_al_aceptante, ctx.accounts.cuenta_de_garantia.amount)?; // el total de tokens almacenados

        // que más podemos hacer??

        /*
        Una vez que se han intercambiado los token_A y token_B entre el usuario inicializador
        y el usuario aceptante, se cierra la cuenta de garantía y la renta se devuelve al 
        usuario inicializador
        */

        let cpi_cerrar = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            CloseAccount {
                account: ctx.accounts.cuenta_de_garantia.to_account_info(),
                destination: ctx.accounts.inicializador.to_account_info(), // se devuelve la renta al usuario incializador
                authority: ctx.accounts.cuenta_de_garantia.to_account_info(),
            },
        ).with_signer(semillas_firma);

        // hacemos cpi al token program para cerrar la PDA que es una cuenta token
        close_account(cpi_cerrar)?;

        Ok(())
    }

}

/*
Inicializar Propuesta de Intercambio

Esta instrucción crea un nuevo intercambio de tokens por parte de un usuario
inicializador. 

cuentas:
- escrow: cuenta que almacena todos los datos del intercambio
- incializador: usuario que crea la propuesta de intercambio
- inicializador_token_account_a: cuenta token propiedad del inicializador, que almacena token_A
- cuenta_de_garantia: cuenta token (PDA) donde se almacenan los token_A que el usuario inicializador
    desea intercambiar.
- token_a: cuenta mint del token_A que el inicializador está dispuesto a intercambiar
- token_b: cuenta mint del token_B que el inicializador desea recibir en el intercambio
- system_program: programa para gestionar cuentas en Solana
- token_program: programa para gestionar tokens en Solana
- rent: variable del sistema de la renta.
*/
#[derive(Accounts)]
#[instruction(id:String)]
pub struct Inicializar<'info> {
    #[account(
    init, // necesitamos crear la cuenta que almacena los datos del intercambio
    payer = inicializador, // el usuario inicializadoor corre con los gastos 
    space = 8 + Escrow::INIT_SPACE,
    // el escrow es una PDA
    seeds= [ // PDA
        inicializador.key().as_ref(),
        id.as_ref()
    ],
    bump,
   )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut, // se le debita saldo
    )]
    pub inicializador: Signer<'info>, // el usuario que crea la propuesta de intercambio

    #[account(
        mut, // se le debitaran saldo de token A
        constraint = inicializador_cuenta_token_a.mint == token_a.key() // verificaciones
    )]
    pub inicializador_cuenta_token_a: Account<'info, TokenAccount>,

    #[account(
        init, // al crear la propuesta de intercambio se crea la cuenta PDA de garantia
        payer = inicializador,
        seeds = [ // PDA
            escrow.key().as_ref()
        ],
        bump,
        token::mint = token_a,
        token::authority = cuenta_de_garantia,
    )]
    pub cuenta_de_garantia: Account<'info, TokenAccount>,

    // tokens
    pub token_a: Account<'info, Mint>,
    pub token_b: Account<'info, Mint>,

    //programas
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    //variables
    pub rent: Sysvar<'info, Rent>,
}

/*
Finalizar Propuesta de Intercambio

Esta instrucción realiza las transferencias necesarias para dar
por finalizado el intercambio de tokens. 

cuentas:
- escrow: cuenta que almacena todos los datos del intercambio
- cuenta_de_garantia: cuenta token (PDA) donde se almacenan los token_A que el usuario inicializador
    desea intercambiar.
- aceptante: usuario que acepta la propuesta de intercambio.
- incializador: usuario que crea la propuesta de intercambio
- inicializador_token_account_b: cuenta token propiedad del inicializador, donde recibira los token_B
- aceptante_token_account_b: cuenta token propiedad del aceptante, que almacena token_B
- aceptante_cuenta_token_a: cuenta token propiedad del aceptante, donde recibira los token_A
- token_program: programa para gestionar tokens en Solana
*/

#[derive(Accounts)]
pub struct Finalizar<'info> {
    #[account(
        mut,
        seeds= [ // PDA
            inicializador.key().as_ref(),
            escrow.id.as_ref()
        ],
        bump = escrow.bump_escrow,
   )]
    pub escrow: Account<'info, Escrow>, // Cuenta de datos que almacena la información del intercambio

    #[account(
        mut, // mutable porque se le debitará saldo
        seeds = [escrow.key().as_ref()], // semillas PDA
        bump = escrow.bump_cuenta_garantia, // bump
    )]
    pub cuenta_de_garantia: Account<'info, TokenAccount>, // almacena los token_A del intercambio

    #[account(mut)]
    pub aceptante: Signer<'info>, // aceptante, el que acepta la oferta del inicializador

    #[account(mut)]
    pub inicializador: SystemAccount<'info>, // el usuario que incializo el intercambio

    // cuentas token
    #[account(
        mut, // mutable porque se le aumentará saldo
        associated_token::mint = escrow.token_b,
        associated_token::authority = escrow.inicializador,
    )]
    pub inicializador_token_account_b: Account<'info, TokenAccount>, // donde el inicializador recibirá token_B

    #[account(
        mut, // mutable porque se le debitará saldo
        associated_token::mint = escrow.token_b,
        associated_token::authority = aceptante.key(),
    )]
    pub aceptante_token_account_b: Account<'info, TokenAccount>, // cuenta token de token_B del aceptante

    #[account(
        mut, // mutable porque se de aumentará saldo
        associated_token::mint = escrow.token_a,
        associated_token::authority = aceptante.key(),
    )]
    pub aceptante_token_account_a: Account<'info, TokenAccount>, // donde el aceptante recibirá token_A

    //programas
    pub token_program: Program<'info, Token>,
}

/* 
Escrow: Cuenta que almacena la información de relacionada a una 
PROPUESTA DE INTERCAMIO de tokens: X cantidad de tokens_A por Y cantidad de tokens_B.

inicializador: usuario que crea una propuesta de intercambio. Ej: 100 USDC por 95 USDT.
token_a: cuenta mint del token_A que el inicializador está dispuesto a intercambiar. Ej: USDC
cantidad_token_a: cantidad de token_A que el incializador desea intercambiar en decimales. Ej: 100 -> 0,0000001
token_b: cuenta mint del token_B que el inicialziador desea recibir. Ej: USDT
cantidad_token_b: cantidad de token_B que el inicializador desea recibir en decimales. Ej: 95 -> 100 -> 0,00000095
bump_escrow: semilla bump para la PDA de la cuenta escrow.
bump_cuenta_garantia: semilla bump para la PDA de la cuentad de garantía.
*/
#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub inicializador: Pubkey, // usuario que crea una propuesta de intercambio
    pub token_a: Pubkey, // mint del token_A que el inicializador está dispuesto a intercambiar
    pub cantidad_token_a: u64, // cantidad de token_A que el incializador desea intercambiar (en decimales)
    pub token_b: Pubkey,       // cuenta mint del token_B que el inicialziador desea recibir
    pub cantidad_token_b: u64, // cantidad de token_B que el inicializador desea recibir (en decimales)

    #[max_len(150)]
    pub id: String, // id del escrow

    // bumps
    pub bump_escrow: u8,          // semilla bump para la PDA de la cuenta escrow
    pub bump_cuenta_garantia: u8, // semilla bump para la PDA de la cuentad de garantía
}
