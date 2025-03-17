use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{Datelike, NaiveDate, NaiveTime};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

// In-memory storage for receipts and their points
static RECEIPTS: Lazy<Mutex<HashMap<String, Receipt>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Item {
    #[serde(rename = "shortDescription")]
    short_description: String,
    price: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Receipt {
    retailer: String,
    #[serde(rename = "purchaseDate")]
    purchase_date: String,
    #[serde(rename = "purchaseTime")]
    purchase_time: String,
    items: Vec<Item>,
    total: String,
    #[serde(skip)]
    points: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReceiptId {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Points {
    points: u32,
}

// Process receipt endpoint
async fn process_receipt(receipt: web::Json<Receipt>) -> impl Responder {
    let mut receipt = receipt.into_inner();
    

    let points = calculate_points(&receipt);
    receipt.points = Some(points);
    

    let id = Uuid::new_v4().to_string();

    RECEIPTS.lock().unwrap().insert(id.clone(), receipt);
    
    HttpResponse::Ok().json(ReceiptId { id })
}

// Get points endpoint
async fn get_points(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    

    let receipts = RECEIPTS.lock().unwrap();
    
    match receipts.get(&id) {
        Some(receipt) => {
            let points = receipt.points.unwrap_or(0);
            HttpResponse::Ok().json(Points { points })
        },
        None => HttpResponse::NotFound().body("No receipt found for that ID.")
    }
}

// Calculate points based on the rules
fn calculate_points(receipt: &Receipt) -> u32 {
    let mut points = 0;
    

    points += receipt.retailer.chars().filter(|c| c.is_alphanumeric()).count() as u32;
    

    if receipt.total.ends_with(".00") {
        points += 50;
    }
    

    if let Ok(total) = receipt.total.parse::<f64>() {
        if (total * 100.0) as u32 % 25 == 0 {
            points += 25;
        }
    }
    

    points += (receipt.items.len() / 2) as u32 * 5;
    

    for item in &receipt.items {
        let trimmed_desc = item.short_description.trim();
        if trimmed_desc.len() % 3 == 0 && !trimmed_desc.is_empty() {
            if let Ok(price) = item.price.parse::<f64>() {
                points += (price * 0.2).ceil() as u32;
            }
        }
    }
    
    if let Ok(total) = receipt.total.parse::<f64>() {
        if total > 10.0 {
            points += 5;
        }
    }
    
    if let Ok(date) = NaiveDate::parse_from_str(&receipt.purchase_date, "%Y-%m-%d") {
        if date.day() % 2 != 0 {
            points += 6;
        }
    }
    
    if let Ok(time) = NaiveTime::parse_from_str(&receipt.purchase_time, "%H:%M") {
        let after_2pm = NaiveTime::parse_from_str("14:00", "%H:%M").unwrap();
        let before_4pm = NaiveTime::parse_from_str("16:00", "%H:%M").unwrap();
        
        if time > after_2pm && time < before_4pm {
            points += 10;
        }
    }
    
    points
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Receipt Processor on http://localhost:8080");
    
    HttpServer::new(|| {
        App::new()
            .route("/receipts/process", web::post().to(process_receipt))
            .route("/receipts/{id}/points", web::get().to(get_points))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
