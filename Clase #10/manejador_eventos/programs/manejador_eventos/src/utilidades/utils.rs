// calcula el % representado por la <cantidad> sobre el <total>
// ejemplo:
// total = 53 tokens del evento
// cantidad = 5 tokens del evento
// porcentaje = (5*100)/53 = 9.43%
pub fn calcular_porcentaje(total: u64, cantidad: u64) -> f64 {
    let temp = cantidad * 100;
    let porcentaje = (temp as f64) / (total as f64);
    porcentaje
}

// calcula el <porcentaje>% del <total> de tokens aceptados
// ejemplo:
// total = 150 tokens aceptados (en la boveda de ganancias)
// porcentaje = 9.43 (9.43%) de colaboracion
// ganacias del colaborador = (150.0)*(9.43)/(100.0).floor() = 33 tokens aceptados
pub fn calcular_ganancias(total: u64, porcentaje: f64) -> u64 {
    let temp = (total as f64) * porcentaje;
    let gannacias = (temp as f64) / (100.);
    gannacias.floor() as u64
}
