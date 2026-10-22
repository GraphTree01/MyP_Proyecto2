// Clase para encapsular operaciones CRUD según el patrón de diseño DAO
struct RolaDao {

    pb fn inserta_rola() -> Rola {};
    
    pb fn get_rola() -> Rola {};

    pb fn lee_rola(rola: Rola) -> ();

    pb fn actualiza_rola(rola: Rola) -> ();

    pb fn elimina_rola(rola: Rola) -> ();
}
