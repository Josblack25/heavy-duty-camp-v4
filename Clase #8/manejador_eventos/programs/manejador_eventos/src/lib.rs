use anchor_lang::prelude::*;
use crate::instrucciones::*;

// treamos los modulos al scope
mod colecciones;
mod instrucciones;
mod utilidades;
// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("5huE81uqJWvRjuJNE4ms1sBX7fnCK58Y2Ywx6PAtea7m");

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

    pub fn eliminar_evento(ctx: Context<EliminarEvento>) -> Result<()> {
        instrucciones::eliminar_evento(ctx)?;
        Ok(())
    }

    pub fn finalizar_evento(ctx: Context<FinalizarEvento>) -> Result<()> {
        instrucciones::finalizar_evento(ctx)?;
        Ok(())
    }

}
