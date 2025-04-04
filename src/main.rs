use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::distributions::{Distribution, WeightedIndex};
use rand::thread_rng;
use serde::Serialize;
use std::error::Error;
use std::fs::File;
use std::io;

#[derive(Serialize)]
struct LotteryResponse {
    number: String,
}

// AppState สำหรับเก็บข้อมูลน้ำหนักที่โหลดมาจาก CSV
struct AppState {
    weights: Vec<u32>,
}

/// ฟังก์ชันโหลดข้อมูลสถิติจากไฟล์ CSV
fn load_statistics(file_path: &str) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut weights = vec![0u32; 10];

    for result in rdr.records() {
        let record = result?;
        let digit: usize = record.get(0).unwrap().trim().parse()?;
        let frequency: u32 = record.get(1).unwrap().trim().parse()?;
        if digit < 10 {
            weights[digit] = frequency;
        }
    }

    // ตรวจสอบว่ามีข้อมูลครบทุกตัวเลข หากตัวใดไม่มีข้อมูล จะเป็น 0
    for (i, w) in weights.iter().enumerate() {
        if *w == 0 {
            eprintln!("Warning: ไม่มีข้อมูลความถี่สำหรับเลข {}", i);
        }
    }
    Ok(weights)
}

/// ฟังก์ชันสุ่มเลข 0-9 โดยใช้น้ำหนักที่ได้จาก state
fn weighted_random_digit(weights: &Vec<u32>) -> u8 {
    let dist = WeightedIndex::new(weights).unwrap();
    let mut rng = thread_rng();
    dist.sample(&mut rng).try_into().unwrap()
}

/// ฟังก์ชันสร้างเลขที่มีจำนวนหลักตามที่ต้องการ
fn generate_number(weights: &Vec<u32>, n_digits: usize) -> String {
    (0..n_digits)
        .map(|_| weighted_random_digit(weights).to_string())
        .collect::<Vec<_>>()
        .join("")
}

/// Endpoint สำหรับรางวัลที่ 1 (เลข 3 ตัวท้าย)
async fn simulate_first_prize(data: web::Data<AppState>) -> impl Responder {
    let number = generate_number(&data.weights, 3);
    HttpResponse::Ok().json(LotteryResponse { number })
}

/// Endpoint สำหรับเลขท้าย 2 ตัว
async fn simulate_last2(data: web::Data<AppState>) -> impl Responder {
    let number = generate_number(&data.weights, 2);
    HttpResponse::Ok().json(LotteryResponse { number })
}

/// Endpoint สำหรับเลขหน้า 3 ตัว
async fn simulate_front3(data: web::Data<AppState>) -> impl Responder {
    let number = generate_number(&data.weights, 3);
    HttpResponse::Ok().json(LotteryResponse { number })
}

/// Endpoint สำหรับเลขท้าย 3 ตัว (เพิ่มเติม)
async fn simulate_last3(data: web::Data<AppState>) -> impl Responder {
    let number = generate_number(&data.weights, 3);
    HttpResponse::Ok().json(LotteryResponse { number })
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    // โหลดสถิติจากไฟล์ CSV เมื่อเริ่มโปรแกรม
    let weights = load_statistics("lottery_stats.csv").unwrap_or_else(|err| {
        eprintln!("Error loading statistics: {}. ใช้ค่า default", err);
        vec![10, 12, 15, 20, 18, 25, 30, 22, 17, 14]
    });
    println!("Loaded weights: {:?}", weights);

    let app_state = web::Data::new(AppState { weights });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/simulate/first_prize", web::get().to(simulate_first_prize))
            .route("/simulate/last2", web::get().to(simulate_last2))
            .route("/simulate/front3", web::get().to(simulate_front3))
            .route("/simulate/last3", web::get().to(simulate_last3))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
