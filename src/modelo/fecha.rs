//! Clase auxiliar que verifica un formato particular de cadenas; fechas. Nos servirá
// para checar las fechas que el usuario puede poner en la intefaz.

use chrono::NaiveDate;

use crate::modelo::error::ErrorModelo;

/// Valida una fecha opcional en formato estricto `AAAA-MM-DD`.
pub fn validar(cadena: Option<&str>) -> Result<Option<String>, ErrorModelo> {
    let Some(cadena) = cadena.map(str::trim).filter(|cadena| !cadena.is_empty()) else {
        return Ok(None);
    };

    let fecha = NaiveDate::parse_from_str(cadena, "%Y-%m-%d").map_err(|_| {
        ErrorModelo::Validacion(format!(
            "La fecha '{cadena}' no es válida; usa el formato AAAA-MM-DD."
        ))
    })?;
    if fecha.format("%Y-%m-%d").to_string() != cadena {
        return Err(ErrorModelo::Validacion(format!(
            "La fecha '{cadena}' no tiene el formato AAAA-MM-DD."
        )));
    }

    Ok(Some(cadena.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acepta_fechas_validas_y_campos_vacios() {
        assert_eq!(
            validar(Some("2000-02-29")).unwrap(),
            Some("2000-02-29".to_owned())
        );
        assert_eq!(validar(None).unwrap(), None);
        assert_eq!(validar(Some(" ")).unwrap(), None);
    }

    #[test]
    fn rechaza_fechas_imposibles_y_formatos_no_validos() {
        assert!(validar(Some("2023-02-29")).is_err());
        assert!(validar(Some("2024-2-03")).is_err());
        assert!(validar(Some("03/02/2024")).is_err());
    }
}
