use anchor_lang::prelude::*;
use anchor_spl::token::{transfer,close_account, CloseAccount, Token, TokenAccount, Transfer, Mint};

declare_id!("D7ko992PKYLDKFy3fWCQsePvWF3Z7CmvoDHnViGf8bfm");

#[program]
pub mod escrow {
    use super::*;

    pub fn initializar(ctx: Context<Inicializar>,token_a_amount: u64,token_b_amount: u64) -> Result<()> {

        // almacenamos la información del intercambio en la cuenta escrow
        ctx.accounts.escrow.inicializador = ctx.accounts.inicializador.key();
        ctx.accounts.escrow.mint_a = ctx.accounts.mint_a.key();
        ctx.accounts.escrow.token_a_amount = token_a_amount;
        ctx.accounts.escrow.mint_b = ctx.accounts.mint_b.key();
        ctx.accounts.escrow.token_b_amount = token_b_amount;
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
                from: ctx.accounts.inicializador_token_account_a.to_account_info(),
                to: ctx.accounts.cuenta_de_garantia.to_account_info(),
                authority: ctx.accounts.inicializador.to_account_info(),
            },
        );

       // hacemos CPI al token program
       transfer(cpi_ctx,token_a_amount)?;

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
                // ya hemos comprobado en el context de la instrucción si quiere enviar los tokens correctos
                from: ctx.accounts.aceptante_token_account_b.to_account_info(),
                to: ctx.accounts.inicializador_token_account_b.to_account_info(),
                // el que acepta la oferta tiene que firmar desde el client
                authority: ctx.accounts.aceptante.to_account_info(),
            },
        );

        transfer(cpi_al_inicializador, ctx.accounts.escrow.token_b_amount)?;

       /*
       Una vez que el usuario incializador recibe los token_B del intercambio, el programa
       tranfiere de la cuenta de garantia (PDA) los token_A hacia la cuenta token
       del usuario aceptante.
        */

        // definimos las semillas firmantes para la PDA
        let semillas_firma: &[&[&[u8]]] = &[&[ctx.accounts.escrow.to_account_info().key.as_ref(),
        &[ctx.accounts.escrow.bump_cuenta_garantia]]];

        // transferimos los tokens de la cuenta de garantía a la cuenta token del usuario aceptante
        let cpi_al_aceptante = CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.cuenta_de_garantia.to_account_info(),
                to: ctx.accounts.aceptante_token_account_a.to_account_info(),
                authority: ctx.accounts.cuenta_de_garantia.to_account_info(),
            },
        ).with_signer(semillas_firma);

        // hacemos CPI al token program
        transfer(cpi_al_aceptante, ctx.accounts.cuenta_de_garantia.amount)?; // el total de tokens almacenados

        /*
        Una vez que se han intercambiado los token_A y token_B entre el usuario inicializador
        y el usuario aceptante, se cierra la cuenta de garantía y la renta se devuelve al 
        usuario inicializador
        */
        let cpi_cerrar= CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            CloseAccount {
                account: ctx.accounts.cuenta_de_garantia.to_account_info(),
                destination: ctx.accounts.inicializador.to_account_info(),
                authority: ctx.accounts.cuenta_de_garantia.to_account_info(),
            },
        ).with_signer(semillas_firma);

        // hacemos cpi al token program apra cerrar la PDA que es una cuenta token
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
- mint_a: cuenta mint del token_A que el inicializador está dispuesto a intercambiar
- mint_b: cuenta mint del token_B que el inicializador desea recibir en el intercambio
- system_program: programa para gestionar cuentas en Solana
- token_program: programa para gestionar tokens en Solana
- rent: variable del sistema de la renta.
*/

#[derive(Accounts)]
pub struct Inicializar<'info> {

    #[account(  init, payer = inicializador, space = 8 + Escrow::INIT_SPACE)]
    pub escrow: Account<'info, Escrow>, // Cuenta de datos que almacena la información del intercambio

    #[account(mut)]
    pub inicializador: Signer<'info>, // El usuraio inicializador firma la transacción 

    #[account(
        mut, // mutable porque se de debitará saldo
        associated_token::mint = mint_b.key(),
        associated_token::authority = inicializador.key(),
    )]
    pub inicializador_token_account_a: Account<'info, TokenAccount>, // Token Account del token A del usuario inicializador

    #[account(
        init, // creamos la cuenta para la PDA
        payer = inicializador, // el usuario inicializado para los fees
        seeds = [
            escrow.key().as_ref(), // utilizamos la dirección de la cuenta escrow como semilla 
        ],
        bump, // bump de la PDA
        token::mint = mint_a, // la PDA almacena token_A
        token::authority = cuenta_de_garantia,  // queremos que el mismo program tenga la autoridad sobre el cuenta de garantía
                                                // por esto se establece como autiridad su propia dirección.
    )]
    pub cuenta_de_garantia: Account<'info, TokenAccount>, // Aquí es donde almacenamos los token_A del inicializador
    

    pub mint_a: Account<'info, Mint>, // necesaria para crear la cuenta de garantía
    pub mint_b: Account<'info, Mint>, // necesaria para alamacenar los datos del intercambio en la cuenta escrow
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


/*
Finalizar Propuesta de Intercambio

Esta instrucción realiza las transferencias necesarias para dar
por finalizado el intercambio de tokens. 

cuentas:
- escrow: cuenta que almacena todos los datos del intercambio
- incializador: usuario que crea la propuesta de intercambio
- inicializador_token_account_a: cuenta token propiedad del inicializador, que almacena token_A
- cuenta_de_garantia: cuenta token (PDA) donde se almacenan los token_A que el usuario inicializador
    desea intercambiar.
- mint_a: cuenta mint del token_A que el inicializador está dispuesto a intercambiar
- mint_b: cuenta mint del token_B que el inicializador desea recibir en el intercambio
- system_program: programa para gestionar cuentas en Solana
- token_program: programa para gestionar tokens en Solana
- rent: variable del sistema de la renta.
*/
#[derive(Accounts)]
pub struct Finalizar<'info> {
    #[account(mut)]
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

    #[account(
        mut, // mutable porque se le aumentará saldo
        associated_token::mint = escrow.mint_b,
        associated_token::authority = escrow.inicializador,
    )]
    pub inicializador_token_account_b: Account<'info, TokenAccount>, // donde el inicializador recibirá token_B

    #[account(
        mut, // mutable porque se le debitará saldo
        associated_token::mint = escrow.mint_b,
        associated_token::authority = aceptante.key(),
    )]
    pub aceptante_token_account_b: Account<'info, TokenAccount>, // cuenta token de token_B del aceptante

    #[account(
        mut, // mutable porque se de aumentará saldo
        associated_token::mint = escrow.mint_a,
        associated_token::authority = aceptante.key(),
    )]  
    pub aceptante_token_account_a: Account<'info, TokenAccount>, // donde el aceptante recibirá token_A

    pub token_program: Program<'info, Token>,
}


/* 
Escrow: Cuenta que almacena la información de relacionada a una 
PROPUESTA DE INTERCAMIO de tokens: X cantidad de tokens_A por Y cantidad de tokens_B.

inicializador: usuario que crea una propuesta de intercambio. Ej: 100 USDC por 95 USDT.
mint_a: cuenta mint del token_A que el inicializador está dispuesto a intercambiar. Ej: USDC
token_a_amount: cantidad de token_A que el incializador desea intercambiar. Ej: 100
mint_b: cuenta mint del token_B que el inicialziador desea recibir. Ej: USDT
token_b_amount: cantidad de token_B que el inicializador desea recibir. Ej: 95
bump_cuenta_garantia: semilla bump para la PDA de la cuentad de garantía.
*/ 
#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub inicializador: Pubkey,      // usuario que crea una propuesta de intercambio
    pub mint_a: Pubkey,             // mint del token_A que el inicializador está dispuesto a intercambiar
    pub token_a_amount: u64,        // cantidad de token_A que el incializador desea intercambiar
    pub mint_b: Pubkey,             // cuenta mint del token_B que el inicialziador desea recibir
    pub token_b_amount: u64,        // cantidad de token_B que el inicializador desea recibir
    pub bump_cuenta_garantia: u8,   // semilla bump para la PDA de la cuentad de garantía
}
