use actix_files as fs;
use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Vaga {
    id: usize,
    numero: usize,
    coberta: bool,
    apartamento: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SorteioData {
    apartamento: String,
    vaga: usize,
}

fn init_vagas() -> Vec<Vaga> {
    let mut vagas = Vec::new();

    for i in 1..=90 {
        let coberta = i <= 30;
        vagas.push(Vaga {
            id: i,
            numero: i,
            coberta,
            apartamento: None,
        });
    }

    vagas
}

async fn obter_vagas(data: web::Data<Arc<Mutex<Vec<Vaga>>>>) -> impl Responder {
    let vagas = data.lock().unwrap();
    HttpResponse::Ok().json(vagas.clone())
}

async fn sortear_vaga(
    data: web::Json<SorteioData>,
    vagas_data: web::Data<Arc<Mutex<Vec<Vaga>>>>,
) -> impl Responder {
    let mut vagas = vagas_data.lock().unwrap();

    if vagas
        .iter()
        .any(|v| v.apartamento == Some(data.apartamento.clone()))
    {
        return HttpResponse::BadRequest().body("Este apartamento já escolheu uma vaga.");
    }

    let vaga_disponivel = vagas
        .iter_mut()
        .find(|v| v.id == data.vaga && v.apartamento.is_none()); // Atualize esta linha

    if let Some(vaga) = vaga_disponivel {
        vaga.apartamento = Some(data.apartamento.clone());
        HttpResponse::Ok().json(vaga.clone())
    } else {
        HttpResponse::BadRequest().body("Todas as vagas já foram ocupadas.")
    }
}

// Função de manipulador para a rota GET na raiz
async fn index() -> actix_web::Result<NamedFile> {
    let path: PathBuf = PathBuf::from("static/index.html");
    Ok(NamedFile::open(path)?)
}

// Registro da rota GET na função main
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let vagas = init_vagas();
    let vagas_data = web::Data::new(Arc::new(Mutex::new(vagas)));

    HttpServer::new(move || {
        App::new()
            .app_data(vagas_data.clone())
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/vagas").route(web::get().to(obter_vagas)))
            .service(web::resource("/sortear").route(web::post().to(sortear_vaga)))
            .service(fs::Files::new("/", "static"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
