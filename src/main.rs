mod appconfig;
mod utils;
mod result;
mod transaction;

use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use appconfig::CONFIG;
use result::TrieResult;
use serde_json::Value;
use utils::bash_cmd;

use crate::transaction::Transaction;

fn main() {
    let num_of_thread = CONFIG.get::<u64>("NUM_OF_THREAD").unwrap();
    let interval = CONFIG.get::<u64>("INTERVAL").unwrap();

    let shared_data = Arc::new(Mutex::new((Instant::now())));

    for t in 0..num_of_thread {
      let shared_data_clone = shared_data.clone();

      thread::spawn(move || {
        loop {
            let now = Instant::now();
            let mut last_processed = shared_data_clone.lock().unwrap();
            
            if now.duration_since(*last_processed) >= Duration::from_secs(interval) {
                // Mark the start of processing
                *last_processed = now;

                // Your processing logic goes here
                process_data(t);

                // Release the lock before sleeping
                drop(last_processed);

                // Sleep for 5 seconds before starting the next iteration
                // thread::sleep(Duration::from_secs(interval));
            }
        }
      });
    }

    // Keep the main thread alive
    thread::park();
}

fn process_data(t: u64) {
    // Your processing code here
    println!("Processing data {t}...");

    let result = get_pending_tx();
    // println!("result: {:?}", result);

    if result.result.is_some() {
      let parse_txs: Result<Vec<Transaction>, _> = serde_json::from_str(&result.result.unwrap());

      match parse_txs {
        Ok(txs) => {
          for tx in txs {
            println!("tx: {:?}", tx);

            let mut validator_method;
            match tx.method.clone().as_str() {
              "contract" => validator_method = Some("validate_contract".to_string()),
              "metadata" => validator_method = Some("validate_metadata".to_string()),
              "cron" => validator_method = Some("validate_cron".to_string()),
              _ => validator_method = None
            }

            if validator_method.is_some() {
              let args = vec![
                validator_method.unwrap(),
                serde_json::to_string(&tx).unwrap(),
              ];

              let result = bash_cmd("CONSENSUS", args);

              println!("result: {:?}", result);
            }
          }
        },
        _ => (),
      }
    }
    // Simulate processing time
    // thread::sleep(Duration::from_secs(2));
    println!("Processing {t} complete.");
}

fn get_pending_tx() -> TrieResult {
  let args = vec!["get_pending_tx".to_string()];
  
  bash_cmd("WORLD_STATE", args).into()

}