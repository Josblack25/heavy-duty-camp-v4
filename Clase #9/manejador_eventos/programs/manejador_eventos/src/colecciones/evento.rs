use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Evento {
    //------------------------- datos del evento------------------------------
    #[max_len(16)] // el id del evento no debe tener más de 16 caracteres
    pub id: String,

    #[max_len(40)] // el nombre del evento no debe tener más de 40 caracteres
    pub nombre: String,

    #[max_len(150)] // la descripcion del evento no debe tener más de 150 caracteres
    pub descripcion: String,

    //--------------------------- precios -----------------------------------------
    pub precio_entrada: u64, // expresado en la unidad minima del token
    pub precio_token: u64,   // expresado en la unidad minima del token

    //-------------------------- estado del evento ---------------------------------
    pub activo: bool,           // indica si el evento esta activo o ya finalizó
    pub total_sponsors: u64,    // total de sponsors que ha tenido el evento
    pub sponsors_actuales: u64, // total de sponsor actuales en el evento
    pub tokens_vendidos: u64,   // total de tokens de sponsor vendidos
    pub entradas_vendidas: u64, // total de entradas vendidas

    //--------------------------- cuentas ---------------------------------------------
    pub autoridad: Pubkey,      // usuario que crea el evento
    pub token_aceptado: Pubkey, // Mint del token aceptado en las transacciones. Ej: USDC

    //------------------------- bumps --------------------------------------------------
    pub bump_evento: u8,           // el evento será una PDA
    pub bump_token_evento: u8, // el token del evento será una PDA (solo el programa puede hacer mint)
    pub bump_boveda_evento: u8, // la boveva del evento será una PDA
    pub bump_boveda_ganancias: u8, // la boveda de ganancias será una PDA
}

//----------------------------- semillas extra para las PDAS ------------------------------
impl Evento {
    pub const SEMILLA_EVENTO: &'static str = "evento"; //semilla para la PDA del evento
    pub const SEMILLA_TOKEN_EVENTO: &'static str = "token_evento"; //semilla para la PDA del token del evento
    pub const SEMILLA_BOVEDA_EVENTO: &'static str = "boveda_evento"; //semilla para la PDA de la boveda del evento
    pub const SEMILLA_BOVEDA_GANANCIAS: &'static str = "boveda_ganancias"; // semilla para la PDA de la boveda de ganancias
}
