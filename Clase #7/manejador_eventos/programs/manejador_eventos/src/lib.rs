use anchor_lang::prelude::*;
use crate::instrucciones::*;

// treamos los modulos al scope
mod colecciones;
mod instrucciones;
// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("3rnFTCvX6kCbWDRHwiNgok2p8TiaCw6mTAPnV2rT35Mt");

#[program]
mod manejador_eventos {
    use super::*;

    pub fn crear_evento(
        ctx: Context<CrearEvento>,
        id: String,
        nombre: String,
        descripcion: String,
        precio_entrada: f64,
        precio_token: f64,
    ) -> Result<()> {
        instrucciones::crear_evento(ctx, id, nombre, descripcion, precio_entrada, precio_token)?;
        Ok(())
    }
}
