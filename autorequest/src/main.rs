use std::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Country {
    id: u32,
    value: String,
    iso_a2: String,
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct State {
    id: u32,
    value: String,
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct City {
    value: String,
    key: String,
    state_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    // Token de autorización
    let bearer_token = "";

    // Datos del país Guatemala
    let country = Country {
        id: 69,
        value: "Guatemala".to_string(),
        iso_a2: "Gtm".to_string(),
        key: "GT".to_string(),
    };

    // Listado de estados y ciudades
    let states = vec![
        ("Guatemala", vec!["Guatemala City", "Antigua Guatemala", "Escuintla"]),
    ];

    let mut state_id_counter = 800; // ID del contador para nuevos estados

    for (state_name, cities) in states {
        // Verificación del estado
        let check_state_url = format!(
            "https://locale-geo-data-production.quadi.io/api/state/{}",
            format!("{}-{}", country.key, &state_name[0..2].to_uppercase())
        );

        let state_exists = client
            .get(&check_state_url)
            .header("Authorization", format!("Bearer {}", bearer_token))
            .send()
            .await;

        let mut state_id = None;
        if let Ok(response) = state_exists {
            if response.status().is_success() {
                // Guardamos la respuesta en una variable para evitar moverla
                let body = response.text().await?;
                println!("Respuesta del estado: {}", body);

                // Intentamos deserializar el JSON directamente
                match response.json::<State>().await {
                    Ok(existing_state) => {
                        state_id = Some(existing_state.id);
                        println!("Estado ya existe: {}", state_name);
                    }
                    Err(e) => {
                        println!("Error de deserialización: {}", e);
                    }
                }
            }
        }

        // Si el estado no existe, crear uno nuevo
        if state_id.is_none() {
            let state = State {
                id: state_id_counter,
                value: state_name.to_string(),
                key: format!("{}-{}", country.key, &state_name[0..2].to_uppercase()),
            };

            let state_url = "https://locale-geo-data-production.quadi.io/api/state";
            let state_response = client
                .post(state_url)
                .header("Authorization", format!("Bearer {}", bearer_token))
                .json(&state)
                .send()
                .await?;

            if state_response.status().is_success() {
                // Obtener el estado recién creado
                let created_state: State = state_response.json().await?;
                state_id = Some(created_state.id); // Asignar el id después de la creación
                println!("Estado creado: {} con id: {}", state_name, created_state.id); // Log de id
            } else {
                println!(
                    "Error al crear estado '{}': {} - {:?}",
                    state_name,
                    state_response.status(),
                    state_response.text().await?
                );
                continue; // Saltar al siguiente estado si no se pudo crear
            }
        }

        // Crear las ciudades solo si el estado fue creado o ya existe
        if let Some(state_id) = state_id {
            let cities_payload: Vec<City> = cities
                .into_iter()
                .map(|city_name| City {
                    value: city_name.to_string(),
                    key: format!(
                        "{}-{}-{}",
                        country.key,
                        &state_name[0..2].to_uppercase(),
                        &city_name[0..3].to_uppercase()
                    ),
                    state_id,
                })
                .collect();

            let city_url = "https://locale-geo-data-production.quadi.io/api/city/";
            let city_response = client
                .post(city_url)
                .header("Authorization", format!("Bearer {}", bearer_token))
                .json(&cities_payload) // Enviamos todas las ciudades en un solo POST
                .send()
                .await?;

            if city_response.status().is_success() {
                println!("Ciudades creadas: {:?}", city_response.text().await?);
            } else {
                println!(
                    "Error al crear ciudades para el estado '{}': {} - {:?}",
                    state_name,
                    city_response.status(),
                    city_response.text().await?
                );
            }
        }

        state_id_counter += 1; // Incrementar el contador de ID para el siguiente estado
    }

    Ok(())
}
