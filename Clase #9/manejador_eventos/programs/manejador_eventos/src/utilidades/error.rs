use anchor_lang::prelude::*;

// definimos una estructura enum para los errores personalizados

// marcamos con laa maccroo de errores
#[error_code]
pub enum CodigoError {
    // texto del error
    #[msg("Solo la autoridad del evento puede eliminarlo.")]
    UsuarioNoAutorizado, // "codigo o tipo del error"

    #[msg("No puedes eliminar un evento con colaboradores")]
    EventoConSponsors,

    #[msg("No puedes eliminar el evento si la bóveda del evento no está vacía")]
    BovedaDelEventoNoVacia,

    #[msg("No puedes eliminar el evento si la bóveda de ganancias no está vacía")]
    BovedaDeGananciasNoVacia,

    #[msg("Los tokens almacenados en la cuneta no corresponden al token esperado")]
    TokenIncorrecto,

    #[msg("La cuenta no tiene fondos suficientes")]
    SaldoInsuficiente,
}
