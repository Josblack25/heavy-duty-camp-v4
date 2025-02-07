use anchor_lang::prelude::*;

// Codigos de error personalizadoss
#[error_code]
pub enum CodigoError {
    #[msg("Solo la autoridad del evento puede eliminarlo.")]
    UsuarioNoAutorizado,

    #[msg("No puedes eliminar un evento con colaboradores")]
    EventoConSponsors,

    #[msg("No puedes eliminar el evento si la bóveda del evento no está vacía")]
    BovedaDelEventoNoVacia,

    #[msg("No puedes eliminar el evento si la bóveda de ganancias no está vacía")]
    BovedaDeGananciasNoVacia,
}
