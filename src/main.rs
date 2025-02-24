use actix_web::{web, post, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::task;
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};


#[derive(Deserialize)]
struct ReportRequest {
    mango_a: f64, // Cambiado de u32 a f64
    mango_b: f64, // Cambiado de u32 a f64
    mango_c: f64,
}

#[derive(Serialize)]
struct ReportResponse {
    message: String,
}

#[derive(Serialize, Deserialize)]
struct ServiceAccountKey {
    #[serde(rename = "type")]
    key_type: String,
    project_id: String,
    private_key_id: String,
    private_key: String,
    client_email: String,
    client_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_x509_cert_url: String,
}

#[derive(Serialize, Deserialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'a str,
    aud: &'a str,
    exp: usize,
    iat: usize,
}
async fn get_firebase_access_token(service_account_key: &ServiceAccountKey) -> Result<String, Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
    let claims = Claims {
        iss: &service_account_key.client_email,
        scope: "https://www.googleapis.com/auth/firebase.messaging",
        aud: &service_account_key.token_uri,
        exp: now + 3600,
        iat: now,
    };

    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(service_account_key.private_key.as_bytes())?;
    let jwt = encode(&header, &claims, &encoding_key)?;
    println!("JWT generado: {}", jwt);


    let client = Client::new();
    let params = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("assertion", &jwt),

    ];
    let res = client.post(&service_account_key.token_uri)
        .form(&params)
        .send()
        .await?;

    let res_json: serde_json::Value = res.json().await?;
    let access_token = res_json["access_token"].as_str().ok_or("Failed to get access token")?;
    Ok(access_token.to_string())
}

async fn send_push_notification(title: &str, body: &str, firebase_token: &str, device_token: &str) {
    let client = Client::new();
    let payload = json!({
        "message": {
            "token": device_token,
            "notification": {
                "title": title,
                "body": body,
            }
        }
    });

    if let Err(err) = client
        .post("https://fcm.googleapis.com/v1/projects/fcmappmovil/messages:send")
        .bearer_auth(firebase_token)
        .json(&payload)
        .send()
        .await
    {
        println!("Error al enviar la notificación: {:?}", err);
    }
}

async fn generate_report(req: web::Json<ReportRequest>) -> impl Responder {
    // Cargar credenciales desde el archivo JSON
    let file = File::open("fcmappmovil-firebase.json").expect("No se pudo abrir el archivo fcm_account.json");
    let reader = BufReader::new(file);
    let service_account_key: ServiceAccountKey = serde_json::from_reader(reader)
        .expect("No se pudieron cargar las credenciales de Firebase");

    // Obtener el token de acceso
    let firebase_token = match get_firebase_access_token(&service_account_key).await {
        Ok(token) => token,
        Err(err) => {
            println!("Error al obtener el token de acceso: {:?}", err);
            return HttpResponse::InternalServerError().body("Error al obtener el token de acceso");
        }
    };

    let device_token = "eyRw4ciQT9mxJQnffEaOMO:APA91bG3QHZTwq_-AQnbOUT9M0sLR-M5VFzph6x4II49B0v3LVRODUqjgsvIqKeKPj34FIdSegD5915866s-MrSTP5yasmmpD3hMXSs2Me1fiNVwkyuSP6I";
    
    

    // Retrasar el envío de la notificación final 10 segundos
    sleep(Duration::from_secs(10)).await;
    // Enviar notificación de inicio
    send_push_notification(
        "Inicio de Proceso",
        "El proceso de empaque ha comenzado.",
        &firebase_token,
        device_token,
    )
    .await;

     // Convertir los valores a u32
     let mango_a = req.mango_a as u32;
     let mango_b = req.mango_b as u32;
     let mango_c = req.mango_c as u32; 

    // Procesar el informe en una tarea intensiva
    let report = task::spawn_blocking(move || {
        generate_report_logic(mango_a, mango_b, mango_c)
    })
    .await
    .unwrap();

    // Retrasar el envío de la notificación final 10 segundos
    sleep(Duration::from_secs(20)).await;

    // Enviar notificación con el informe
    send_push_notification(
        "Reporte de Empaque Finalizado",
        &report.message,
        &firebase_token,
        device_token,
    )
    .await;

    HttpResponse::Ok().json(report)
}

fn generate_report_logic(mango_a: u32, mango_b: u32, mango_c: u32) -> ReportResponse {
    let cajas_por_paleta = 240;
    let paletas_por_contenedor = 20;

    let paletas_a = mango_a / cajas_por_paleta;
    let sobrantes_a = mango_a % cajas_por_paleta;

    let paletas_b = mango_b / cajas_por_paleta;
    let sobrantes_b = mango_b % cajas_por_paleta;

    let paletas_c = mango_c / cajas_por_paleta;
    let sobrantes_c = mango_c % cajas_por_paleta;

    let total_paletas = paletas_a + paletas_b + paletas_c;
    let faltantes = if total_paletas >= paletas_por_contenedor {
        0
    } else {
        paletas_por_contenedor - total_paletas
    };

    ReportResponse {
        message: format!(
            "Reporte:\n\
             - Mango A: {} paleta(s), {} cajas sobrantes\n\
             - Mango B: {} paleta(s), {} cajas sobrantes\n\
             - Mango C: {} paleta(s), {} cajas sobrantes\n\
             Total: {} paleta(s)\n\
             Faltan {} paleta(s) para completar un contenedor.",
            paletas_a, sobrantes_a, paletas_b, sobrantes_b, paletas_c, sobrantes_c, total_paletas, faltantes
        ),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/generate_report", web::post().to(generate_report))
    })
    .bind("0.0.0.0:3030")?
    .run()
    .await
}